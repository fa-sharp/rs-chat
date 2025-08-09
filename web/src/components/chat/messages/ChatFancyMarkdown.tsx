import type { ReactNode } from "@tanstack/react-router";
import svelteHighlight from "highlight.svelte";
import { common } from "lowlight";
import { Check, ChevronDown, ChevronUp, Copy } from "lucide-react";
import {
  type ComponentProps,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import remarkGfm from "remark-gfm";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

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
  const [isExpanded, setIsExpanded] = useState(false);
  const [showExpandButton, setShowExpandButton] = useState(false);

  const handleCopy = async () => {
    if (ref.current) {
      try {
        await navigator.clipboard.writeText(ref.current.innerText.trim());
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
      } catch (error) {
        console.error("Failed to copy text:", error);
      }
    }
  };

  const toggleExpanded = () => {
    setIsExpanded(!isExpanded);
  };

  const checkIfExpandButtonNeeded = useCallback(() => {
    if (ref.current) {
      const maxHeight =
        parseFloat(getComputedStyle(ref.current).lineHeight) * 12; // 12 lines
      const actualHeight = ref.current.scrollHeight;
      setShowExpandButton(actualHeight > maxHeight);
    }
  }, []);

  useEffect(() => {
    if (children) checkIfExpandButtonNeeded();
  }, [children, checkIfExpandButtonNeeded]);

  if (!children) return null;
  return (
    // `not-prose` disables the Tailwind typography styles
    <div className="not-prose relative">
      <Button
        aria-label="Copy code"
        className="absolute top-1.5 right-1.5 z-[999] size-8 text-muted-foreground opacity-75 hover:opacity-100 focus-visible:opacity-100"
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
      <pre
        ref={ref}
        className={cn(
          "text-xs md:text-sm overflow-x-auto rounded-md transition-all",
          !isExpanded &&
            showExpandButton &&
            "max-h-[11lh] overflow-hidden mask-b-from-85%",
        )}
        style={{ lineHeight: "1.5" }}
      >
        {children}
      </pre>
      {showExpandButton && (
        <Button
          aria-label={isExpanded ? "Collapse code" : "Expand code"}
          onClick={toggleExpanded}
          variant="outline"
          size="icon"
          className="absolute bottom-4 left-1/2 transform -translate-x-1/2 dark:bg-secondary dark:hover:bg-secondary rounded-full text-xs opacity-80 hover:opacity-100 focus-visible:opacity-100"
        >
          {isExpanded ? (
            <ChevronUp className="size-5" />
          ) : (
            <ChevronDown className="size-5" />
          )}
        </Button>
      )}
    </div>
  );
}
