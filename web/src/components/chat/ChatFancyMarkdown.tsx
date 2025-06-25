import type { ReactNode } from "@tanstack/react-router";
import svelteHighlight from "highlight.svelte";
import { common } from "lowlight";
import { Check, Copy } from "lucide-react";
import { type ComponentProps, useRef, useState } from "react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import remarkGfm from "remark-gfm";

import { Button } from "../ui/button";

/**
 * Markdown with plugins for syntax highlighting, line numbers, copying code, etc.
 * Should be asynchronously imported as it loads a bunch of languages.
 */
export default function ChatFancyMarkdown({
  children,
}: {
  children: ReactNode;
}) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={rehypePlugins}
      components={components}
    >
      {children}
    </Markdown>
  );
}

const rehypePlugins: ComponentProps<typeof Markdown>["rehypePlugins"] = [
  [
    rehypeHighlight,
    {
      languages: {
        ...common,
        svelte: svelteHighlight,
      },
    },
  ],
  [rehypeHighlightCodeLines, { showLineNumbers: true }],
];

const components: ComponentProps<typeof Markdown>["components"] = {
  pre: CodeWrapper,
};

/** Wrapper for code blocks and added copy button */
function CodeWrapper({ children }: { children?: ReactNode }) {
  const ref = useRef<HTMLPreElement>(null);
  const [isCopied, setIsCopied] = useState(false);

  const handleCopy = async () => {
    if (ref.current) {
      try {
        await navigator.clipboard.writeText(
          ref.current.innerText
            .slice(5)
            .trim(), // Slice off the text of the copy button
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
    <div className="not-prose relative">
      {/* `not-prose` disables the Tailwind typography styles */}
      <Button
        aria-label="Copy code"
        className="absolute top-1 right-1 size-8 text-muted-foreground opacity-75 hover:opacity-100 focus-visible:opacity-100"
        onClick={handleCopy}
        variant="outline"
        size="icon"
      >
        {isCopied ? (
          <Check className="size-5 text-green-600" />
        ) : (
          <Copy className="size-4" />
        )}
      </Button>
      <pre ref={ref} className="text-xs md:text-sm overflow-x-auto">
        {children}
      </pre>
    </div>
  );
}
