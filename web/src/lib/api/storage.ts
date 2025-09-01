import { useMutation, useQueryClient } from "@tanstack/react-query";

import type { components } from "./types";

const uploadFile = async ({
  sessionId,
  path,
  file,
}: {
  sessionId: string;
  path: string;
  file: File;
}) => {
  const res = await fetch(`/storage/${sessionId}/${path}`, {
    method: "POST",
    body: file,
  });
  if (!res.ok) {
    throw new Error(
      `Failed to upload file: ${res.status} ${(await res.json()).message}`,
    );
  }

  return res.json() as Promise<components["schemas"]["ChatRsFile"]>;
};

export const useUploadFile = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: uploadFile,
    onSettled: (_data, _err, vars) =>
      queryClient.invalidateQueries({
        queryKey: ["files", { sessionId: vars.sessionId }],
      }),
  });
};
