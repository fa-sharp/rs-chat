import { Check, File, FileText, Image, Paperclip, Upload } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { useDropzone } from "react-dropzone";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useSessionFiles, useUploadFile } from "@/lib/api/storage";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import ChatSettingsBadge from "./ChatSettingsBadge";

interface FileSelectionDialogProps {
  sessionId: string;
  selectedFiles: components["schemas"]["ChatRsFile"][];
  onAddFile: (file: components["schemas"]["ChatRsFile"]) => void;
  onRemoveFile: (fileId: string) => void;
  onRemoveAllFiles: () => void;
  onOpenChange?: (open: boolean) => void;
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
  onOpenChange,
}: FileSelectionDialogProps) {
  const { data: files, isLoading } = useSessionFiles(sessionId);
  const { mutate: uploadFile } = useUploadFile();

  const [open, setOpen] = useState(false);
  const handleOpenChange = useCallback(
    (open: boolean) => {
      setOpen(open);
      onOpenChange?.(open);
    },
    [onOpenChange],
  );

  const [uploadingFiles, setUploadingFiles] = useState<string[]>([]);

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

  const selectedCount = selectedFiles.length;
  const totalCount = fileList.length;

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
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
        <div
          {...getRootProps()}
          className={`relative ${
            isDragActive ? "ring-2 ring-primary ring-offset-2 bg-primary/5" : ""
          } transition-all duration-200 rounded-lg`}
        >
          <input {...getInputProps()} />
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
          <DialogHeader>
            <DialogTitle>Attach Files</DialogTitle>
            <DialogDescription>Attach files to your message.</DialogDescription>
          </DialogHeader>

          <div className="mt-1 space-y-4">
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

            {uploadingFiles.length > 0 && (
              <div className="text-xs text-muted-foreground flex items-center gap-1 bg-muted/50 px-2 py-1 rounded">
                <Upload className="size-3 animate-pulse" />
                Uploading {uploadingFiles.length} file
                {uploadingFiles.length > 1 ? "s" : ""}...
              </div>
            )}

            <div className="max-h-80 overflow-y-auto">
              {isLoading ? (
                <div className="flex items-center justify-center py-8 text-muted-foreground">
                  Loading files...
                </div>
              ) : totalCount === 0 ? (
                <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                  <File className="size-8 mb-2 opacity-50" />
                  <p className="text-sm">
                    Drag and drop files here to upload them
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
              <Button onClick={() => handleOpenChange(false)}>
                {selectedCount > 0 ? `Attach Files` : "Done"}
              </Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
