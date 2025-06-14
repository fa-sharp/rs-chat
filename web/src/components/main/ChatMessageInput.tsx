import { CornerDownLeft, Mic } from "lucide-react";
import { useCallback, useRef, useState, type FormEventHandler } from "react";
import { Button } from "../ui/button";
import { ChatInput } from "../ui/chat/chat-input";

interface Props {
  isGenerating: boolean;
  onSubmit: (message: string) => void;
}

export default function ChatMessageInput({ isGenerating, onSubmit }: Props) {
  const formRef = useRef<HTMLFormElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const [enterKeyShouldSubmit, setEnterKeyShouldSubmit] = useState(true);

  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (
        (enterKeyShouldSubmit && e.key === "Enter" && !e.shiftKey) ||
        (!enterKeyShouldSubmit && e.key === "Enter" && e.shiftKey)
      ) {
        e.preventDefault();
        if (isGenerating || !inputRef.current?.value) return;

        onSubmit(inputRef.current?.value);
        formRef.current?.reset();
      }
    },
    [isGenerating, onSubmit, enterKeyShouldSubmit],
  );

  const handleSubmit: FormEventHandler<HTMLFormElement> = useCallback(
    (ev) => {
      ev.preventDefault();
      if (!inputRef.current?.value) return;

      onSubmit(inputRef.current?.value);
      formRef.current?.reset();
    },
    [onSubmit],
  );

  return (
    <form
      ref={formRef}
      onSubmit={handleSubmit}
      className="flex flex-col gap-2 relative rounded-lg border bg-background focus-within:ring-1 focus-within:ring-ring"
    >
      <ChatInput
        ref={inputRef}
        onKeyDown={onKeyDown}
        placeholder="Type your message here..."
        className="rounded-lg bg-background border-0 shadow-none focus-visible:ring-0"
      />
      <div className="flex items-center gap-2 p-3 pt-0">
        <Button
          type="button"
          variant={enterKeyShouldSubmit ? "default" : "outline"}
          size="icon"
          title="Toggle Enter key behavior"
          onClick={() => setEnterKeyShouldSubmit((prev) => !prev)}
        >
          <CornerDownLeft className="size-3.5" />
          <span className="sr-only">Toggle Enter key</span>
        </Button>

        <Button type="button" variant="ghost" size="icon">
          <Mic className="size-4" />
          <span className="sr-only">Use Microphone</span>
        </Button>

        <Button
          disabled={isGenerating}
          type="submit"
          size="sm"
          className="ml-auto gap-1.5 flex items-center"
        >
          Send Message
          {!enterKeyShouldSubmit && <kbd>Shift + </kbd>}
          <CornerDownLeft className="size-3.5" />
        </Button>
      </div>
    </form>
  );
}
