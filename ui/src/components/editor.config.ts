import * as monaco from "monaco-editor/esm/vs/editor/editor.api";

export const ID_LANGUAGE_SQL = "sql";

export const COMMAND_CONFIG: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: "--",
    blockComment: ["/*", "*/"],
  },
  brackets: [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
  ],
  autoClosingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: "(", close: ")" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  wordPattern: /(-?\d*\.\d\w*)|([a-zA-Z_]\w*)/g,
  indentationRules: {
    increaseIndentPattern: /(\{|\[|\()/,
    decreaseIndentPattern: /(\}|\]|\))/,
  },
};

export const autoSuggestionCompletionItems = (
  range: monaco.languages.CompletionItem["range"],
): monaco.languages.CompletionList => {
  const _suggestions = [
    {
      label: "SELECT",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "SELECT ",
      range,
    },
    {
      label: "FROM",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "FROM ",
      range,
    },
    {
      label: "WHERE",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "WHERE ",
      range,
    },
    {
      label: "GROUP BY",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "GROUP BY ",
      range,
    },
    {
      label: "HAVING",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "HAVING ",
      range,
    },
    {
      label: "ORDER BY",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "ORDER BY ",
      range,
    },
    {
      label: "LIMIT",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "LIMIT ",
      range,
    },
    {
      label: "AND",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "AND ",
      range,
    },
    {
      label: "OR",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "OR ",
      range,
    },
    {
      label: "NOT",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "NOT ",
      range,
    },
    {
      label: "BETWEEN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "BETWEEN ",
      range,
    },
    {
      label: "IN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "IN ",
      range,
    },
    {
      label: "LIKE",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "LIKE ",
      range,
    },
    {
      label: "IS NULL",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "IS NULL ",
      range,
    },
    {
      label: "IS NOT NULL",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "IS NOT NULL ",
      range,
    },
    {
      label: "INNER JOIN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "INNER JOIN ",
      range,
    },
    {
      label: "LEFT JOIN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "LEFT JOIN ",
      range,
    },
    {
      label: "RIGHT JOIN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "RIGHT JOIN ",
      range,
    },
    {
      label: "FULL JOIN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "FULL JOIN ",
      range,
    },
    {
      label: "ON",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "ON ",
      range,
    },
    {
      label: "AS",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "AS ",
      range,
    },
    {
      label: "COUNT",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "COUNT()",
      range,
    },
    {
      label: "SUM",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "SUM()",
      range,
    },
    {
      label: "AVG",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "AVG()",
      range,
    },
    {
      label: "MIN",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "MIN()",
      range,
    },
    {
      label: "MAX",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "MAX()",
      range,
    },
    {
      label: "CAST",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "CAST()",
      range,
    },
    {
      label: "DATE",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "DATE()",
      range,
    },
    {
      label: "NOW",
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: "NOW()",
      range,
    },
    {
      label: "JOIN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "JOIN ",
      range,
    },
    {
      label: "INSERT INTO",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "INSERT INTO ",
      range,
    },
    {
      label: "UPDATE",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "UPDATE ",
      range,
    },
    {
      label: "DELETE",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "DELETE ",
      range,
    },
    {
      label: "CREATE TABLE",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "CREATE TABLE ",
      range,
    },
    {
      label: "DROP TABLE",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "DROP TABLE ",
      range,
    },
    {
      label: "PRAGMA",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "PRAGMA ",
      range,
    },
    {
      label: "VACUUM",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "VACUUM;",
      range,
    },
    {
      label: "ATTACH DATABASE",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "ATTACH DATABASE '' AS '';",
      range,
    },
    {
      label: "SERIAL",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "SERIAL ",
      range,
    },
    {
      label: "RETURNING",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "RETURNING ",
      range,
    },
    {
      label: "CREATE EXTENSION",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "CREATE EXTENSION ",
      range,
    },
    {
      label: "AUTO_INCREMENT",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "AUTO_INCREMENT ",
      range,
    },
    {
      label: "ENGINE",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "ENGINE=",
      range,
    },
    {
      label: "SHOW DATABASES",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "SHOW DATABASES;",
      range,
    },
    {
      label: "SHOW TABLES",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "SHOW TABLES;",
      range,
    },
    {
      label: "COPY",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "COPY ",
      range,
    },
    {
      label: "EXPORT DATABASE",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "EXPORT DATABASE '';",
      range,
    },
    {
      label: "IMPORT DATABASE",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "IMPORT DATABASE '';",
      range,
    },
    {
      label: "CREATE MATERIALIZED VIEW",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "CREATE MATERIALIZED VIEW ",
      range,
    },
    {
      label: "OPTIMIZE TABLE",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "OPTIMIZE TABLE ",
      range,
    },
    {
      label: "ALTER TABLE",
      kind: monaco.languages.CompletionItemKind.Snippet,
      insertText: "ALTER TABLE ",
      range,
    },
    {
      label: "EXPLAIN",
      kind: monaco.languages.CompletionItemKind.Keyword,
      insertText: "EXPLAIN ",
      range,
    },
  ];

  // Remove duplicates from suggestions using filter method
  const suggestions = _suggestions.filter(
    (item, index, self) =>
      self.findIndex((t) => t.label === item.label) === index,
  );

  return { suggestions };
};
