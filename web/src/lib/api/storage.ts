import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
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
  const res = await fetch(`/api/storage/${sessionId}/${path}`, {
    method: "POST",
    body: file,
  });
  if (!res.ok) {
    throw new Error(`Failed to upload file: ${(await res.json()).message}`);
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

export const useSessionFiles = (sessionId: string) =>
  useQuery({
    queryKey: ["files", { sessionId }],
    queryFn: async () => {
      const res = await client.GET("/storage/{session_id}", {
        params: { path: { session_id: sessionId } },
      });
      if (res.error) {
        throw new Error(`Failed to fetch files: ${res.error.message}`);
      }
      return res.data;
    },
  });
