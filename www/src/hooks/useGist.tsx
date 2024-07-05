import { useState, useEffect } from "react";

const useGist = (defaultParam: string) => {
  const [content, setContent] = useState<string>("");

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    let gistParam = params.get("gist");

    if (!gistParam) {
      gistParam = defaultParam;
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
          console.info(`loaded gist: ${gistParam}`);
        } else {
          console.warn("gist didn't have any content");
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
