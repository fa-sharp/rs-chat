import type { ReactNode } from "@tanstack/react-router";
import { Check, Copy } from "lucide-react";
import { useRef, useState } from "react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import remarkGfm from "remark-gfm";
import { Button } from "../ui/button";

/** Markdown with plugins for syntax highlighting, line numbers, copying code, etc. */
export default function ChatFancyMarkdown({
  children,
}: {
  children: ReactNode;
}) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[
        rehypeHighlight,
        [rehypeHighlightCodeLines, { showLineNumbers: true }],
      ]}
      components={{
        pre: CodeWrapper,
      }}
    >
      {children}
    </Markdown>
  );
}

/** Wrapper for code blocks and added copy button */
function CodeWrapper({ children }: { children?: ReactNode }) {
  const ref = useRef<HTMLPreElement>(null);
  const [isCopied, setIsCopied] = useState(false);

  const handleCopy = async () => {
    if (ref.current) {
      try {
        await navigator.clipboard.writeText(
          ref.current.innerText.slice(5).trim(), // Slice off the text of the copy button
        );
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
      } catch (error) {
        console.error("Failed to copy text:", error);
      }
    }
  };

  if (!children) return null;
  return (
    <div className="not-prose">
      {/* `not-prose` disables the Tailwind typography styles */}
      <pre ref={ref} className="relative">
        <Button
          className="absolute top-2 right-2 opacity-85 hover:opacity-100"
          onClick={handleCopy}
          variant="outline"
          size="sm"
        >
          {isCopied ? (
            <Check className="size-4 text-green-600" />
          ) : (
            <Copy className="size-3" />
          )}
          Copy
        </Button>
        {children}
      </pre>
    </div>
  );
}
