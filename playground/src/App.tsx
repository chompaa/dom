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
    <main className="flex h-screen flex-row bg-white">
      <section className="relative flex w-full flex-col border-r-2 border-gray-100">
        <div className="flex h-12 flex-row items-center gap-2 bg-gray-50 text-gray-500">
          <button
            className="m-1 border-2 border-gray-500 bg-gray-100 px-2 hover:bg-gray-500 hover:text-gray-100"
            onClick={run}
          >
            run
          </button>
        </div>
        <div className="flex flex-1">
          <Editor editorRef={editorRef} />
        </div>
      </section>
      <section className="relative flex max-h-full w-full flex-col overflow-x-scroll border-l-2 border-gray-100 bg-white">
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
  );
};

export default App;
