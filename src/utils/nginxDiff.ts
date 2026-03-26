import type { ServerBlock } from '@/types/config';
import type { FileChangeLogDetail, FileChangeScopeDiff } from '@/types/nginx';

interface ParsedLocationBlock {
  key: string;
  label: string;
  raw: string;
}

const DISABLED_SERVER_BEGIN_MARKER = '# nginx-config-manager managed-disabled-server begin';
const DISABLED_SERVER_END_MARKER = '# nginx-config-manager managed-disabled-server end';

const normalizeLines = (content: string) => content.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

const commentOutLine = (line: string) => (line.length === 0 ? '#' : `# ${line}`);

const buildLocationKey = (label: string, occurrences: Map<string, number>) => {
  const currentCount = occurrences.get(label) ?? 0;
  occurrences.set(label, currentCount + 1);
  return `${label}__${currentCount}`;
};

const findBlockEnd = (lines: string[], start: number) => {
  let depth = 0;
  let foundStart = false;

  for (let index = start; index < lines.length; index += 1) {
    for (const char of lines[index]) {
      if (char === '{') {
        depth += 1;
        foundStart = true;
      } else if (char === '}') {
        depth -= 1;
        if (foundStart && depth === 0) {
          return index;
        }
      }
    }
  }

  return start;
};

const parseLocationLabel = (line: string) => {
  const parts = line.trim().replace(/\{$/, '').trim().split(/\s+/);
  if (parts.length < 2) {
    return 'location';
  }

  if (parts.length >= 3) {
    return `${parts[1]} ${parts[2]}`.trim();
  }

  return parts[1];
};

const extractLocationBlocks = (serverText: string): ParsedLocationBlock[] => {
  const lines = normalizeLines(serverText).split('\n');
  const locations: ParsedLocationBlock[] = [];
  const occurrences = new Map<string, number>();

  let index = 0;
  while (index < lines.length) {
    const trimmed = lines[index].trim();
    if (!trimmed.startsWith('location') || !trimmed.includes('{')) {
      index += 1;
      continue;
    }

    const end = findBlockEnd(lines, index);
    const label = parseLocationLabel(trimmed);
    locations.push({
      key: buildLocationKey(label, occurrences),
      label,
      raw: lines.slice(index, end + 1).join('\n'),
    });
    index = end + 1;
  }

  return locations;
};

export const buildLocationDiffs = (
  beforeServerText: string,
  afterServerText: string,
): FileChangeScopeDiff[] => {
  const beforeLocations = new Map(
    extractLocationBlocks(beforeServerText).map((location) => [location.key, location]),
  );
  const afterLocations = new Map(
    extractLocationBlocks(afterServerText).map((location) => [location.key, location]),
  );

  const locationKeys = new Set([...beforeLocations.keys(), ...afterLocations.keys()]);

  return Array.from(locationKeys)
    .map((key) => {
      const before = beforeLocations.get(key);
      const after = afterLocations.get(key);
      const beforeRaw = before?.raw ?? '';
      const afterRaw = after?.raw ?? '';

      if (beforeRaw === afterRaw) {
        return null;
      }

      return {
        label: after?.label ?? before?.label ?? 'location',
        before: beforeRaw,
        after: afterRaw,
      } satisfies FileChangeScopeDiff;
    })
    .filter((diff): diff is FileChangeScopeDiff => Boolean(diff));
};

export const renderManagedServerBlock = (serverText: string, enabled: boolean) => {
  if (enabled) {
    return normalizeLines(serverText);
  }

  const commentedLines = normalizeLines(serverText)
    .split('\n')
    .map(commentOutLine);
  return [DISABLED_SERVER_BEGIN_MARKER, ...commentedLines, DISABLED_SERVER_END_MARKER].join('\n');
};

export const extractLineRange = (content: string, startLine: number, endLine: number) => {
  const lines = normalizeLines(content).split('\n');
  return lines.slice(Math.max(0, startLine - 1), Math.max(0, endLine)).join('\n');
};

export const removeLineRange = (content: string, startLine: number, endLine: number) => {
  const lines = normalizeLines(content).split('\n');
  return [...lines.slice(0, Math.max(0, startLine - 1)), ...lines.slice(Math.max(0, endLine))].join('\n');
};

export const buildServerScopeLabel = (server: ServerBlock | null, fallback = '目标 Server') => {
  if (!server) {
    return fallback;
  }

  if (server.category?.trim()) {
    return server.category.trim();
  }

  if (server.serverName.length > 0) {
    return server.serverName.join(', ');
  }

  if (server.listen.length > 0) {
    return `listen ${server.listen.join(', ')}`;
  }

  return fallback;
};

export const buildFileChangeSummary = (
  operationLabel: string,
  configPath: string,
  targetLabel?: string,
) => {
  const normalizedPath = configPath.split(/[\\/]/).filter(Boolean).pop() || configPath;
  return targetLabel
    ? `${operationLabel} · ${targetLabel} · ${normalizedPath}`
    : `${operationLabel} · ${normalizedPath}`;
};

export const createFileChangeDetail = ({
  operationLabel,
  configPath,
  fileBefore,
  fileAfter,
  serverDiff,
  locationDiffs,
}: {
  operationLabel: string;
  configPath: string;
  fileBefore: string;
  fileAfter: string;
  serverDiff?: FileChangeScopeDiff | null;
  locationDiffs?: FileChangeScopeDiff[];
}): FileChangeLogDetail => ({
  operationLabel,
  configPath,
  savedAt: Date.now(),
  fileDiff: {
    label: '文件',
    before: normalizeLines(fileBefore),
    after: normalizeLines(fileAfter),
  },
  serverDiff: serverDiff
    ? {
        label: serverDiff.label,
        before: normalizeLines(serverDiff.before),
        after: normalizeLines(serverDiff.after),
      }
    : null,
  locationDiffs: (locationDiffs ?? []).map((diff) => ({
    label: diff.label,
    before: normalizeLines(diff.before),
    after: normalizeLines(diff.after),
  })),
});
