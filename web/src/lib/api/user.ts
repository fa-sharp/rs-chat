import { useQuery } from "@tanstack/react-query";

import { client } from "./client";

export async function getUser() {
  const response = await client.GET("/auth/user");
  if (response.error) {
    throw new Error(response.error.message);
  }
  if (!response.response.ok) {
    throw new Error("Failed to fetch user");
  }
  return response.data;
}

export const useGetUser = () =>
  useQuery({
    queryKey: ["user"],
    queryFn: getUser,
  });

export async function deleteAccount({ confirm }: { confirm: string }) {
  const response = await client.DELETE("/auth/user/delete-my-account", {
    body: { confirm },
  });
  if (response.error) {
    throw new Error(response.error.message);
  }
  return response.data;
}
