const findServerStartIndex = (lines: string[]): number =>
  lines.findIndex((line) => {
    const trimmed = line.trim();
    return trimmed.startsWith('server') && trimmed.includes('{');
  });

const findCategoryLineIndex = (lines: string[], serverStartIndex: number): number => {
  for (let index = serverStartIndex + 1; index < lines.length; index += 1) {
    const trimmed = lines[index].trim();

    if (!trimmed) {
      continue;
    }

    if (trimmed === '}') {
      return -1;
    }

    if (trimmed.startsWith('#')) {
      return index;
    }

    return -1;
  }

  return -1;
};

const resolveCategoryIndent = (
  lines: string[],
  serverStartIndex: number,
  categoryLineIndex: number
): string => {
  if (categoryLineIndex >= 0) {
    const matched = lines[categoryLineIndex].match(/^(\s*)/);
    return matched?.[1] ?? '    ';
  }

  for (let index = serverStartIndex + 1; index < lines.length; index += 1) {
    const trimmed = lines[index].trim();

    if (!trimmed || trimmed === '}') {
      continue;
    }

    const matched = lines[index].match(/^(\s*)/);
    return matched?.[1] ?? '    ';
  }

  return '    ';
};

export const applyServerCategoryToContent = (
  serverText: string,
  categoryName: string
): string => {
  const lines = serverText.split(/\r?\n/);
  const serverStartIndex = findServerStartIndex(lines);

  if (serverStartIndex < 0) {
    return serverText;
  }

  const categoryLineIndex = findCategoryLineIndex(lines, serverStartIndex);
  const nextCategory = categoryName.trim();

  if (!nextCategory) {
    if (categoryLineIndex >= 0) {
      lines.splice(categoryLineIndex, 1);
    }
    return lines.join('\n');
  }

  const nextLine = `${resolveCategoryIndent(lines, serverStartIndex, categoryLineIndex)}# ${nextCategory}`;

  if (categoryLineIndex >= 0) {
    lines[categoryLineIndex] = nextLine;
  } else {
    lines.splice(serverStartIndex + 1, 0, nextLine);
  }

  return lines.join('\n');
};

export const getCategoryLineDelta = (
  originalCategoryName?: string | null,
  nextCategoryName?: string | null
): number => {
  const originalCount = originalCategoryName?.trim() ? 1 : 0;
  const nextCount = nextCategoryName?.trim() ? 1 : 0;

  return nextCount - originalCount;
};
