import { Check, File, FileText, Image, Paperclip } from "lucide-react";
import { useCallback, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useSessionFiles } from "@/lib/api/storage";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import ChatSettingsBadge from "./ChatSettingsBadge";

interface FileSelectionDialogProps {
  sessionId: string;
  selectedFiles: components["schemas"]["ChatRsFile"][];
  onAddFile: (file: components["schemas"]["ChatRsFile"]) => void;
  onRemoveFile: (fileId: string) => void;
  onRemoveAllFiles: () => void;
}

function getFileIcon(fileType: components["schemas"]["ChatRsFileType"]) {
  switch (fileType) {
    case "image":
      return <Image className="size-4" />;
    case "pdf":
      return <FileText className="size-4" />;
    default:
      return <File className="size-4" />;
  }
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`;
}

export default function ChatFileSelect({
  sessionId,
  selectedFiles,
  onAddFile,
  onRemoveFile,
  onRemoveAllFiles,
}: FileSelectionDialogProps) {
  const [open, setOpen] = useState(false);
  const { data: files, isLoading } = useSessionFiles(sessionId, open);

  const fileList = useMemo(() => files || [], [files]);

  const handleFileToggle = useCallback(
    (file: components["schemas"]["ChatRsFile"], isSelected: boolean) => {
      if (isSelected) {
        onRemoveFile(file.id);
      } else {
        onAddFile(file);
      }
    },
    [onAddFile, onRemoveFile],
  );

  const handleSelectAll = useCallback(() => {
    fileList.forEach((file) => {
      if (!selectedFiles.some((f) => f.id === file.id)) {
        onAddFile(file);
      }
    });
  }, [fileList, selectedFiles, onAddFile]);

  const handleDeselectAll = useCallback(() => {
    onRemoveAllFiles();
  }, [onRemoveAllFiles]);

  const selectedCount = selectedFiles.length;
  const totalCount = fileList.length;

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          type="button"
          variant="outline"
          size="icon"
          title="Attach files"
        >
          {selectedCount > 0 && (
            <ChatSettingsBadge>{selectedCount}</ChatSettingsBadge>
          )}
          <Paperclip className="size-3.5" />
          <span className="sr-only">Attach files</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Attach Files</DialogTitle>
          <DialogDescription>
            Select files from your session to attach to your message.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {totalCount > 0 && (
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">
                {selectedCount} of {totalCount} files selected
              </span>
              <div className="flex gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleSelectAll}
                  disabled={selectedCount === totalCount}
                >
                  Select All
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleDeselectAll}
                  disabled={selectedCount === 0}
                >
                  Deselect All
                </Button>
              </div>
            </div>
          )}

          <div className="max-h-80 overflow-y-auto pr-4">
            {isLoading ? (
              <div className="flex items-center justify-center py-8 text-muted-foreground">
                Loading files...
              </div>
            ) : fileList.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <File className="size-8 mb-2 opacity-50" />
                <p className="text-sm">No files in this session</p>
                <p className="text-xs mt-1">
                  Upload files by dragging them to the chat input
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                {fileList.map((file) => {
                  const isSelected = selectedFiles.some(
                    (f) => f.id === file.id,
                  );
                  return (
                    <button
                      key={file.id}
                      type="button"
                      className={cn(
                        "flex items-center justify-between p-3 rounded-lg border cursor-pointer transition-colors hover:bg-muted/50 w-full text-left",
                        isSelected && "bg-primary/5 border-primary/20",
                      )}
                      onClick={() => handleFileToggle(file, isSelected)}
                    >
                      <div className="flex items-center gap-3 min-w-0 flex-1">
                        <div className="flex-shrink-0">
                          {getFileIcon(file.file_type)}
                        </div>
                        <div className="min-w-0 flex-1">
                          <p className="text-sm font-medium truncate">
                            {file.path}
                          </p>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <span>{file.file_type.toUpperCase()}</span>
                            <span>•</span>
                            <span>{formatFileSize(file.size)}</span>
                          </div>
                        </div>
                      </div>
                      <div className="flex-shrink-0 ml-3">
                        {isSelected ? (
                          <div className="flex items-center justify-center size-5 bg-primary text-primary-foreground rounded">
                            <Check className="size-3" />
                          </div>
                        ) : (
                          <div className="size-5 border border-muted-foreground/20 rounded" />
                        )}
                      </div>
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          <div className="flex justify-end gap-2">
            <Button onClick={() => setOpen(false)}>
              {selectedCount > 0 ? `Attach Files` : "Done"}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
