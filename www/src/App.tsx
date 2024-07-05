import Ansi from "ansi-to-react";
import { init_miette_hook, init_panic_hook, interpret } from "dom-wasm";
import { editor } from "monaco-editor";
import { useEffect, useRef, useState } from "react";
import "./App.css";
import Editor from "./components/Editor";
import TabItem from "./components/TabItem";
import TabList from "./components/TabList";
import useOutput from "./hooks/useOutput";

const App = () => {
  const SOURCE = "https://github.com/chompaa/dom";
  const editorRef = useRef<null | editor.IStandaloneCodeEditor>(null);
  const [output, writeOutput, clearOutput] = useOutput();
  const [ast, setAst] = useState("");

  console.log = (text) => {
    writeOutput(text);
  };

  const run = () => {
    clearOutput();
    if (!editorRef.current) {
      return;
    }
    const ast = interpret(editorRef.current.getValue());
    setAst(ast);
  };

  useEffect(() => {
    init_miette_hook();
    init_panic_hook();
  }, []);

  return (
    <div className="flex h-screen max-h-screen flex-col overflow-hidden">
      <section className="flex items-center justify-between bg-zinc-800 py-1 pl-2 text-zinc-50">
        <h1>dom playground</h1>
        <a
          className="mr-1 border-2 border-zinc-100 bg-zinc-800 px-2 hover:bg-zinc-100 hover:text-zinc-800"
          href={SOURCE}
          target="_blank"
        >
          source
        </a>
      </section>
      <main className="flex h-full flex-row bg-white">
        <section className="relative flex w-full flex-col border-r-2 border-zinc-100">
          <div className="flex h-12 flex-row items-center gap-2 bg-zinc-50 text-zinc-800">
            <button
              className="m-1 border-2 border-zinc-800 bg-zinc-100 px-2 hover:bg-zinc-800 hover:text-zinc-100"
              onClick={run}
            >
              run
            </button>
          </div>
          <div className="flex flex-1">
            <Editor editorRef={editorRef} />
          </div>
        </section>
        <section className="relative flex max-h-full w-full flex-col overflow-x-scroll border-l-2 border-zinc-100 bg-white">
          <TabList activeTabIndex={0}>
            <TabItem label="output">
              <pre>
                <Ansi className="font-primary text-2xl">{output}</Ansi>
              </pre>
            </TabItem>
            <TabItem label="ast">
              <pre className="font-primary text-2xl">{ast}</pre>
            </TabItem>
          </TabList>
        </section>
      </main>
    </div>
  );
};

export default App;
