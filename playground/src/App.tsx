import { interpret, set_hook } from "dom-wasm";
import { editor } from "monaco-editor";
import { useRef, useState, useEffect } from "react";
import "./App.css";
import Editor from "./components/Editor";
import TabItem from "./components/TabItem";
import TabList from "./components/TabList";
import useOutput from "./hooks/useOutput";
import Ansi from "ansi-to-react";

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
    set_hook();
  }, []);

  return (
    <main className="h-screen flex flex-row bg-white">
      <section className="w-full relative border-r-2 border-gray-100 flex flex-col">
        <div className=" h-12 bg-gray-50 flex flex-row text-gray-500 items-center gap-2">
          <button
            className="bg-gray-100 border-gray-500 hover:bg-gray-500 hover:text-gray-100 border-2 px-2 m-1"
            onClick={run}
          >
            run
          </button>
        </div>
        <div className="flex flex-1">
          <Editor editorRef={editorRef} />
        </div>
      </section>
      <section className="flex flex-col w-full max-h-full relative bg-white border-l-2 border-gray-100 overflow-x-scroll">
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
