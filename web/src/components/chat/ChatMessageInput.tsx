import { CornerDownLeft, Paperclip, Upload, X } from "lucide-react";
import {
  type FormEventHandler,
  memo,
  useCallback,
  useMemo,
  useState,
} from "react";
import { useDropzone } from "react-dropzone";

import { Button } from "@/components/ui/button";
import { ChatInput } from "@/components/ui/chat/chat-input";
import { useIsMobile } from "@/hooks/use-mobile";
import type { useChatInputState } from "@/hooks/useChatInputState";
import { useCancelChatStream } from "@/lib/api/chat";
import { useProviders } from "@/lib/api/provider";
import { useUploadFile } from "@/lib/api/storage";
import { useTools } from "@/lib/api/tool";
import {
  ChatFileSelect,
  ChatModelSelect,
  ChatMoreSettings,
  ChatProviderSelect,
  ChatToolSelect,
} from "./settings";

/** Handles submitting the user message, along with the current provider/model selection and other settings */
export default memo(function ChatMessageInput({
  inputState,
}: {
  inputState: ReturnType<typeof useChatInputState>;
}) {
  const { data: providers } = useProviders();
  const { data: tools } = useTools();
  const isMobile = useIsMobile();

  const {
    providerId,
    modelId,
    sessionId,
    toolInput,
    files,
    maxTokens,
    temperature,
    error,
    inputRef,
    formRef,
    isGenerating,
    onSelectModel,
    onSetSystemTool,
    onToggleExternalApiTool,
    onAddFile,
    onRemoveFile,
    onRemoveAllFiles,
    setMaxTokens,
    setTemperature,
    canGetAgenticResponse,
    onSubmitUserMessage,
    onSubmitWithoutUserMessage,
  } = inputState;

  const currentProvider = useMemo(() => {
    return providers?.find((p) => p.id === providerId);
  }, [providers, providerId]);

  const setCurrentModel = useCallback(
    (model: string) => onSelectModel(providerId, model),
    [providerId, onSelectModel],
  );

  const [enterKeyShouldSubmit, setEnterKeyShouldSubmit] = useState(true);
  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (
        (enterKeyShouldSubmit && e.key === "Enter" && !e.shiftKey) ||
        (!enterKeyShouldSubmit && e.key === "Enter" && e.shiftKey)
      ) {
        e.preventDefault();
        onSubmitUserMessage();
      }
    },
    [enterKeyShouldSubmit, onSubmitUserMessage],
  );

  const handleFormSubmit: FormEventHandler<HTMLFormElement> = useCallback(
    (ev) => {
      ev.preventDefault();
      onSubmitUserMessage();
    },
    [onSubmitUserMessage],
  );

  const { mutate: onCancel, isPending: isCancelling } =
    useCancelChatStream(sessionId);
  const handleCancel = useCallback(() => {
    onCancel();
  }, [onCancel]);

  const { mutate: uploadFile } = useUploadFile();
  const [uploadingFiles, setUploadingFiles] = useState<string[]>([]);

  const handleFileUpload = useCallback(
    (files: File[]) => {
      if (!sessionId) return;

      const fileNames = files.map((f) => f.name);
      setUploadingFiles((prev) => [...prev, ...fileNames]);

      files.forEach((file) => {
        uploadFile(
          {
            sessionId,
            path: file.name,
            file,
          },
          {
            onSettled: () => {
              setUploadingFiles((prev) =>
                prev.filter((name) => name !== file.name),
              );
            },
            onSuccess: (file) => onAddFile(file),
            onError: (error) => {
              console.error(`Failed to upload ${file.name}:`, error);
            },
          },
        );
      });
    },
    [sessionId, uploadFile, onAddFile],
  );

  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      handleFileUpload(acceptedFiles);
    },
    [handleFileUpload],
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    noClick: true,
    noKeyboard: true,
  });

  return (
    <div
      {...getRootProps()}
      className={`relative ${
        isDragActive ? "ring-2 ring-primary ring-offset-2 bg-primary/5" : ""
      } transition-all duration-200 rounded-lg`}
    >
      <input {...getInputProps()} />
      <form
        ref={formRef}
        onSubmit={handleFormSubmit}
        className="flex flex-col gap-2 relative rounded-lg border bg-background focus-within:ring-1 focus-within:ring-ring"
      >
        {isDragActive && (
          <div className="absolute inset-0 flex items-center justify-center bg-primary/10 backdrop-blur-sm rounded-lg z-10 border-2 border-dashed border-primary">
            <div className="text-center">
              <Upload className="mx-auto h-8 w-8 text-primary mb-2" />
              <p className="text-sm font-medium text-primary">
                Drop files here to upload
              </p>
            </div>
          </div>
        )}
        <ChatInput
          ref={inputRef}
          onKeyDown={onKeyDown}
          placeholder="Type your message..."
          className="rounded-lg bg-background text-foreground border-0 shadow-none focus-visible:ring-0"
        />
        <div className="flex flex-wrap items-center gap-2 p-3 pt-0">
          <ChatProviderSelect
            onSelectModel={onSelectModel}
            currentProvider={currentProvider}
            providers={providers}
          />
          {sessionId &&
            currentProvider &&
            currentProvider.provider_type !== "lorem" && (
              <>
                <ChatModelSelect
                  providerId={providerId}
                  currentModelId={modelId}
                  onSelect={setCurrentModel}
                />
                <ChatMoreSettings
                  currentMaxTokens={maxTokens}
                  currentTemperature={temperature}
                  onSelectMaxTokens={setMaxTokens}
                  onSelectTemperature={setTemperature}
                />
                <ChatToolSelect
                  tools={tools}
                  toolInput={toolInput}
                  onSetSystemTool={onSetSystemTool}
                  onToggleExternalApiTool={onToggleExternalApiTool}
                />
                <ChatFileSelect
                  sessionId={sessionId}
                  selectedFiles={files}
                  onAddFile={onAddFile}
                  onRemoveFile={onRemoveFile}
                  onRemoveAllFiles={onRemoveAllFiles}
                />
                {files.length > 0 && (
                  <div className="flex items-center gap-1 text-xs text-muted-foreground bg-muted/50 px-2 py-1 rounded">
                    <Paperclip className="size-3" />
                    <span>{files.map((file) => file.path).join(", ")}</span>
                  </div>
                )}
                {uploadingFiles.length > 0 && (
                  <div className="text-xs text-muted-foreground flex items-center gap-1">
                    <Upload className="size-3 animate-pulse" />
                    Uploading...
                  </div>
                )}
              </>
            )}

          {error && (
            <div className="text-sm text-destructive-foreground">{error}</div>
          )}

          <div className="ml-auto flex gap-2 items-center">
            {canGetAgenticResponse && (
              <Button
                type="button"
                size="sm"
                disabled={isGenerating}
                onClick={onSubmitWithoutUserMessage}
              >
                Get Agent Response
              </Button>
            )}
            {isGenerating && (
              <Button
                type="button"
                size="sm"
                variant="destructive"
                onClick={handleCancel}
                loading={isCancelling}
                disabled={isCancelling}
              >
                {!isCancelling && <X className="size-4" />}
                Stop
              </Button>
            )}
            <Button
              disabled={isGenerating || uploadingFiles.length > 0}
              type="submit"
              size="sm"
              className="gap-1.5 flex items-center"
            >
              Send Message
              {!enterKeyShouldSubmit && <kbd> Shift + </kbd>}
              <CornerDownLeft className="size-3.5" />
            </Button>
            {!isMobile && (
              <Button
                type="button"
                variant="outline"
                size="icon"
                title="Toggle Enter key behavior"
                onClick={() => setEnterKeyShouldSubmit((prev) => !prev)}
              >
                <CornerDownLeft className="size-3.5" />
                <span className="sr-only">Toggle Enter key</span>
              </Button>
            )}
          </div>
        </div>
      </form>
    </div>
  );
});
