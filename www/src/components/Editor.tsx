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
  const DEFAULT_GIST = "79a9be2dc55a81c3555eaaab1b600228";

  const useGist = async (): Promise<string> => {
    const params = new URLSearchParams(window.location.search);
    let gistParam = params.get("gist");

    if (!gistParam) {
      // If no gist query was provided, use the default one
      gistParam = DEFAULT_GIST;
    }

    const gistUrl = `https://api.github.com/gists/${gistParam}`;

    const response = await fetch(gistUrl);
    if (!response.ok) {
      console.log("gist network response was not ok");
      return "";
    }

    const data = await response.json();
    const file = Object.values(data["files"])[0] as { content: string };

    console.info(`loaded gist: ${gistParam}`);
    return file.content;
  };

  const handleEditorDidMount: OnMount = async (editor, _) => {
    editorRef.current = editor;
    const content = await useGist();
    editor.setValue(content);
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
      defaultValue={""}
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
