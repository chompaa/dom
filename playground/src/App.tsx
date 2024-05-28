import { interpret } from "dom-wasm";
import { editor } from "monaco-editor";
import { useRef } from "react";
import "./App.css";
import Editor from "./components/Editor";
import useOutput from "./hooks/useOutput";

const App = () => {
  const editorRef = useRef<null | editor.IStandaloneCodeEditor>(null);
  const [output, writeOutput, clearOutput] = useOutput();

  console.log = (text) => {
    writeOutput(text);
  };

  const run = () => {
    clearOutput();
    if (!editorRef.current) {
      return;
    }
    interpret(editorRef.current.getValue());
  };

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
      <section className="w-full relative bg-white border-l-2 border-gray-100">
        <div className="flex items-center bg-gray-50 text-gray-500 pl-1 h-12">
          <span>output</span>
        </div>
        <div className="p-1">
          {output.split("\n").map((str, index) => (
            <p key={index}>{str}</p>
          ))}
        </div>
      </section>
    </main>
  );
};

export default App;
