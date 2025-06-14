import { CornerDownLeft, Mic } from "lucide-react";
import {
  useCallback,
  useState,
  type ChangeEventHandler,
  type FormEventHandler,
} from "react";
import { Button } from "../ui/button";
import { ChatInput } from "../ui/chat/chat-input";

interface Props {
  isGenerating: boolean;
  onSubmit: (message: string) => void;
}

export default function ChatMessageInput({ isGenerating, onSubmit }: Props) {
  const [inputValue, setInputValue] = useState("");
  const [enterKeyShouldSubmit, setEnterKeyShouldSubmit] = useState(true);

  const handleInputChange: ChangeEventHandler<HTMLTextAreaElement> =
    useCallback((ev) => {
      setInputValue(ev.target.value);
    }, []);

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (
      (enterKeyShouldSubmit && e.key === "Enter" && !e.shiftKey) ||
      (!enterKeyShouldSubmit && e.key === "Enter" && e.shiftKey)
    ) {
      e.preventDefault();
      if (isGenerating || !inputValue) return;
      onSubmit(e.currentTarget.value);
      setInputValue("");
    }
  };

  const handleSubmit: FormEventHandler<HTMLFormElement> = useCallback(
    (ev) => {
      ev.preventDefault();
      if (!inputValue) return;
      onSubmit(inputValue);
      setInputValue("");
    },
    [inputValue],
  );
  return (
    <form
      // ref={formRef}
      onSubmit={handleSubmit}
      className="flex flex-col gap-2 relative rounded-lg border bg-background focus-within:ring-1 focus-within:ring-ring"
    >
      <ChatInput
        value={inputValue}
        onKeyDown={onKeyDown}
        onChange={handleInputChange}
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
          // disabled={!input || isLoading}
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
