import { SSE, type SSEvent, type ReadyStateEvent } from "sse.js";
import type { components } from "./types";

/** Stream a chat via SSE, using the `eventsource` library */
export function streamChat(
  sessionId: string,
  input: components["schemas"]["SendChatInput"],
  {
    onPart,
    onError,
  }: {
    onPart: (part: string) => void;
    onError: (error: string) => void;
  },
) {
  const source = new SSE(`/api/chat/${sessionId}`, {
    method: "POST",
    payload: JSON.stringify(input),
    headers: { "Content-Type": "application/json" },
  });

  return {
    get readyState() {
      return source.readyState;
    },
    start: new Promise<void>((resolve, reject) => {
      const chatListener = (event: SSEvent) => {
        onPart(event.data);
      };
      const errorListener = (event: SSEvent & { responseCode?: number }) => {
        console.error("Error while streaming:", event);
        if (event.responseCode) {
          let data;
          try {
            data = JSON.parse(event.data).message;
          } catch (error) {
            data = event.data;
          }
          if (typeof data === "string") {
            onError(data);
          } else {
            switch (event.responseCode) {
              case 404:
                onError("Not Found Error");
                break;
              case 500:
                onError("Internal Server Error");
                break;
              default:
                onError(`Error code ${event.responseCode}`);
                break;
            }
          }
          reject();
        } else {
          onError(
            typeof event.data === "string" ? event.data : "Unknown error",
          );
        }
      };

      const endListener = (event: ReadyStateEvent) => {
        if (event.readyState === SSE.CLOSED) {
          source.removeEventListener("chat", chatListener);
          source.removeEventListener("error", errorListener);
          source.removeEventListener("readystatechange", endListener);
          resolve();
        }
      };

      source.addEventListener("chat", chatListener);
      source.addEventListener("error", errorListener);
      source.addEventListener("readystatechange", endListener);
    }),
  };
}
