import {
  BeforeMount,
  Editor as MonacoEditor,
  OnMount,
} from "@monaco-editor/react";
import { editor } from "monaco-editor";
import React from "react";

const Editor = ({
  editorRef,
}: {
  editorRef: React.MutableRefObject<null | editor.IStandaloneCodeEditor>;
}) => {
  const defaultValue = `use std/io
use std/list

fn show_monotonic_lists(end) {
    if end < 0 {
        return
    }

    let result = []
    let count = 1

    loop {
        if count >= end {
            break
        }

        result
        |> list.push(count)
        |> io.print()

        count = count + 1
    }
}

show_monotonic_lists(10)`;

  const handleEditorDidMount: OnMount = (editor, _) => {
    editorRef.current = editor;
  };

  const handleEditorWillMount: BeforeMount = (monaco) => {
    monaco.languages.register({ id: "dom" });
    const keywords = ["if", "let", "fn", "return", "loop", "continue", "break"];

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
        { token: "keyword", foreground: "9d0006", fontStyle: "bold" },
        { token: "variable", foreground: "3c3836" },
        { token: "number", foreground: "8f3f71" },
        { token: "boolean", foreground: "8f3f71" },
        { token: "string", foreground: "79740e" },
        { token: "comment", foreground: "928374" },
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
      defaultValue={defaultValue}
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
