import {
  BeforeMount,
  Editor as MonacoEditor,
  OnMount,
} from "@monaco-editor/react";
import { editor } from "monaco-editor";
import React from "react";
import useGist from "../hooks/useGist";

const Editor = ({
  editorRef,
}: {
  editorRef: React.MutableRefObject<null | editor.IStandaloneCodeEditor>;
}) => {
  const [content] = useGist("79a9be2dc55a81c3555eaaab1b600228");

  const handleEditorDidMount: OnMount = (editor, _) => {
    editorRef.current = editor;
  };

  const handleEditorWillMount: BeforeMount = (monaco) => {
    monaco.languages.register({ id: "dom" });
    const keywords = [
      "if",
      "let",
      "fn",
      "return",
      "loop",
      "continue",
      "break",
      "use",
    ];

    monaco.languages.setMonarchTokensProvider("dom", {
      keywords,
      tokenizer: {
        root: [
          [
            /@?[a-zA-Z][\w$]*/,
            {
              cases: {
                "@keywords": "keyword",
                "true|false": "boolean",
                "@default": "variable",
              },
            },
          ],
          [/\b\d+\b/, "number"],
          [/".*?"/, "string"],
          [/\/\/.*/, "comment"],
          [/\|>/, "pipe"],
        ],
      },
    });

    monaco.languages.setLanguageConfiguration("dom", {
      autoClosingPairs: [
        { open: "{", close: "}" },
        { open: "[", close: "]" },
        { open: "(", close: ")" },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
      ],
      brackets: [
        ["{", "}"],
        ["[", "]"],
        ["(", ")"],
      ],
      surroundingPairs: [
        { open: "{", close: "}" },
        { open: "[", close: "]" },
        { open: "(", close: ")" },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
      ],
      indentationRules: {
        increaseIndentPattern: new RegExp("^.*\\{[^}\"']*$"),
        decreaseIndentPattern: new RegExp("^\\s*\\}"),
      },
      colorizedBracketPairs: [],
    });

    monaco.editor.defineTheme("gruvbox-light", {
      base: "vs",
      inherit: true,
      rules: [
        { token: "keyword", foreground: "9d0006" },
        { token: "variable", foreground: "3c3836" },
        { token: "number", foreground: "8f3f71" },
        { token: "boolean", foreground: "8f3f71" },
        { token: "string", foreground: "79740e" },
        { token: "comment", foreground: "928374" },
        { token: "pipe", foreground: "427b58" },
      ],
      colors: {
        "editor.foreground": "#3c3836",
        "editorCursor.foreground": "#3c3836",
        "editorWhitespace.foreground": "#7c6f6420",
        "editorLineNumber.foreground": "#bdae93",
        "editorLineNumber.activeForeground": "#bdae93",
      },
    });
  };

  return (
    <MonacoEditor
      defaultLanguage="dom"
      defaultValue={content}
      onMount={handleEditorDidMount}
      beforeMount={handleEditorWillMount}
      theme="gruvbox-light"
      options={{
        fontFamily: "Iosevka",
        fontSize: 24,
        minimap: { enabled: false },
        wordWrap: "on",
        scrollbar: {
          vertical: "hidden",
          horizontal: "hidden",
        },
        overviewRulerLanes: 0,
        guides: {
          indentation: false,
        },
        autoClosingBrackets: "always",
        autoIndent: "full",
        bracketPairColorization: {
          enabled: false,
        },
        matchBrackets: "never",
      }}
    />
  );
};

export default Editor;
