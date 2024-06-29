import { useState } from "react";

const useOutput = () => {
  const [output, setOutput] = useState("");

  const writeOutput = (text: string) => {
    setOutput((output) => output.concat(text, "\n"));
  };

  const clearOutput = () => {
    setOutput("");
  };

  return [output, writeOutput, clearOutput] as const;
};

export default useOutput;
