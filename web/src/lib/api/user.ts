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

export async function deleteAccount() {
  const response = await fetch(
    `${import.meta.env.VITE_API_URL || ""}/api/oauth/user/delete-but-only-if-you-are-sure`,
    { method: "DELETE" },
  );
  if (!response.ok) {
    throw new Error(await response.json());
  }
  return await response.text();
}
