import { useState, useEffect } from "react";

const useGist = (param?: string) => {
  const [content, setContent] = useState<string | null>(null);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    let gistParam = params.get("gist");

    if (!gistParam) {
      if (param) {
        gistParam = param;
      } else {
        return;
      }
    }
    const gistUrl = `https://api.github.com/gists/${gistParam}`;

    const fetchData = async () => {
      try {
        const response = await fetch(gistUrl);
        if (!response.ok) {
          console.log("network response was not ok");
        }
        const data = await response.json();
        const file = Object.values(data["files"])[0] as Object;
        if ("content" in file) {
          const content = file.content as string;
          setContent(content);
        }
      } catch (err) {
        console.error(err);
      }
    };

    fetchData();
  }, []);

  return [content];
};

export default useGist;
