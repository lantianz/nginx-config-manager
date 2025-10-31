/**
 * Nginx 配置文件格式化工具
 * 基于 nginxbeautifier 项目改编
 * 原项目: https://github.com/vasilevich/nginxbeautifier
 */

interface ExtractedText {
  filteredInput: string;
  extracted: Record<string, string>;
  getRestored: () => string;
}

interface FormatOptions {
  indentation?: string;
  dontJoinCurlyBracket?: boolean;
  trailingBlankLines?: boolean;
}

/**
 * 提取两个分隔符之间的文本
 */
function extractTextBySeperator(input: string, seperator1: string, seperator2?: string): string {
  if (seperator2 === undefined) seperator2 = seperator1;
  const seperator1Regex = new RegExp(seperator1);
  const seperator2Regex = new RegExp(seperator2);
  const catchRegex = new RegExp(seperator1 + '(.*?)' + seperator2);
  if (seperator1Regex.test(input) && seperator2Regex.test(input)) {
    const match = input.match(catchRegex);
    return match ? match[1] : '';
  }
  return '';
}

/**
 * 提取所有可能的文本（处理引号内的内容）
 */
function extractAllPossibleText(input: string, seperator1: string, seperator2?: string): ExtractedText {
  if (seperator2 === undefined) seperator2 = seperator1;
  const extracted: Record<string, string> = {};
  let textInBetween: string;
  let cnt = 0;
  const seperator1CharCode = seperator1.length > 0 ? seperator1.charCodeAt(0) : '';
  const seperator2CharCode = seperator2.length > 0 ? seperator2.charCodeAt(0) : '';

  while ((textInBetween = extractTextBySeperator(input, seperator1, seperator2)) !== '') {
    const placeHolder = `#$#%#$#placeholder${cnt}${seperator1CharCode}${seperator2CharCode}#$#%#$#`;
    extracted[placeHolder] = seperator1 + textInBetween + seperator2;
    input = input.replace(extracted[placeHolder], placeHolder);
    cnt++;
  }

  return {
    filteredInput: input,
    extracted: extracted,
    getRestored: function () {
      let textToFix = this.filteredInput;
      for (const key in extracted) {
        textToFix = textToFix.replace(key, extracted[key]);
      }
      return textToFix;
    },
  };
}

/**
 * 去除行首尾空格，并将多个空格替换为单个空格
 */
function stripLine(singleLine: string): string {
  const trimmed = singleLine.trim();
  const removedDoubleQuatations = extractAllPossibleText(trimmed, '"', '"');

  // 替换多个空格为单个空格，但跳过 sub_filter 指令
  if (!removedDoubleQuatations.filteredInput.includes('sub_filter')) {
    removedDoubleQuatations.filteredInput = removedDoubleQuatations.filteredInput.replace(/\s\s+/g, ' ');
  }

  return removedDoubleQuatations.getRestored();
}

/**
 * 清理配置行
 */
function cleanLines(configContents: string): string[] {
  const splittedByLines = configContents.split(/\r\n|\r|\n/g);
  let newline = 0;

  for (let index = 0; index < splittedByLines.length; index++) {
    splittedByLines[index] = splittedByLines[index].trim();

    if (!splittedByLines[index].startsWith('#') && splittedByLines[index] !== '') {
      newline = 0;
      let line = (splittedByLines[index] = stripLine(splittedByLines[index]));

      if (
        line !== '}' &&
        line !== '{' &&
        !(line.includes("('{") || line.includes("}')") || line.includes("'{'") || line.includes("'}'"))
      ) {
        const startOfComment = line.indexOf('#');
        // const comment = startOfComment >= 0 ? line.slice(startOfComment) : '';
        let code = startOfComment >= 0 ? line.slice(0, startOfComment) : line;

        const removedDoubleQuatations = extractAllPossibleText(code, '"', '"');
        code = removedDoubleQuatations.filteredInput;

        // 处理右花括号
        const startOfParanthesis = code.indexOf('}');
        if (startOfParanthesis >= 0) {
          if (startOfParanthesis > 0) {
            splittedByLines[index] = stripLine(code.slice(0, startOfParanthesis - 1));
            splittedByLines.splice(index + 1, 0, '}');
          }
          const l2 = stripLine(code.slice(startOfParanthesis + 1));
          if (l2 !== '') splittedByLines.splice(index + 2, 0, l2);
          code = splittedByLines[index];
        }

        // 处理左花括号
        const endOfParanthesis = code.indexOf('{');
        if (endOfParanthesis >= 0) {
          splittedByLines[index] = stripLine(code.slice(0, endOfParanthesis));
          splittedByLines.splice(index + 1, 0, '{');
          const l2 = stripLine(code.slice(endOfParanthesis + 1));
          if (l2 !== '') splittedByLines.splice(index + 2, 0, l2);
        }

        removedDoubleQuatations.filteredInput = splittedByLines[index];
        line = removedDoubleQuatations.getRestored();
        splittedByLines[index] = line;
      }
    } else if (splittedByLines[index] === '') {
      // 移除超过两个的空行
      if (newline++ >= 2) {
        splittedByLines.splice(index, 1);
        index--;
      }
    }
  }

  return splittedByLines;
}

/**
 * 将左花括号与指令放在同一行
 */
function joinOpeningBracket(lines: string[], options: FormatOptions): string[] {
  if (options.dontJoinCurlyBracket) {
    return lines;
  }

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line === '{') {
      if (i >= 1) {
        lines[i] = lines[i - 1] + ' {';
        if (options.trailingBlankLines && lines.length > i + 1 && lines[i + 1].length > 0) {
          lines.splice(i + 1, 0, '');
        }
        lines.splice(i - 1, 1);
      }
    }
  }

  return lines;
}

/**
 * 执行缩进
 */
function performIndentation(lines: string[], indentation: string): string[] {
  const indentedLines: string[] = [];
  let currentIndent = 0;

  for (const line of lines) {
    // 如果是右花括号，减少缩进
    if (!line.startsWith('#') && /.*?\}(\s*#.*)?$/.test(line) && currentIndent > 0) {
      currentIndent -= 1;
    }

    // 添加缩进
    if (line !== '') {
      indentedLines.push(indentation.repeat(currentIndent) + line);
    } else {
      indentedLines.push('');
    }

    // 如果是左花括号，增加缩进
    if (!line.startsWith('#') && /.*?\{(\s*#.*)?$/.test(line)) {
      currentIndent += 1;
    }
  }

  return indentedLines;
}

/**
 * 移除所有空行
 * @param lines - 配置行数组
 * @returns 处理后的行数组（不包含任何空行）
 */
function removeAllBlankLines(lines: string[]): string[] {
  return lines.filter(line => line.trim() !== '');
}

/**
 * 确保文件结尾有且只有一个换行符
 * @param content - 配置内容
 * @returns 处理后的内容
 */
function ensureTrailingNewline(content: string): string {
  // 移除结尾的所有空白字符和换行符
  content = content.replace(/\s+$/, '');

  // 添加一个换行符
  return content + '\n';
}

/**
 * 格式化 Nginx 配置文件
 * @param content - 原始配置内容
 * @param options - 格式化选项
 * @returns 格式化后的配置内容
 */
export function formatNginxConfig(content: string, options: FormatOptions = {}): string {
  try {
    // 设置默认选项
    const indentation = options.indentation || '    '; // 默认 4 个空格
    const formatOptions: FormatOptions = {
      indentation: indentation,
      dontJoinCurlyBracket: options.dontJoinCurlyBracket || false,
      trailingBlankLines: options.trailingBlankLines || false,
    };

    // 1. 清理行
    let lines = cleanLines(content);

    // 2. 将左花括号与指令放在同一行
    lines = joinOpeningBracket(lines, formatOptions);

    // 3. 执行缩进
    lines = performIndentation(lines, indentation);

    // 4. 移除所有空行
    lines = removeAllBlankLines(lines);

    // 5. 合并为字符串
    let result = lines.join('\n');

    // 6. 确保文件结尾有且只有一个换行符
    result = ensureTrailingNewline(result);

    return result;
  } catch (error) {
    console.error('格式化 Nginx 配置时出错:', error);
    throw new Error(`格式化失败: ${error instanceof Error ? error.message : String(error)}`);
  }
}

/**
 * 格式化 Nginx server 块
 * @param serverBlock - server 块内容
 * @returns 格式化后的 server 块
 */
export function formatNginxServerBlock(serverBlock: string): string {
  return formatNginxConfig(serverBlock, {
    indentation: '    ',
    dontJoinCurlyBracket: false,
    trailingBlankLines: false,
  });
}

