import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, useRouter } from "@tanstack/react-router";
import { AlertTriangle, GlobeLock, Trash2, User } from "lucide-react";
import { useState } from "react";

import { Discord, GitHub, Google } from "@/components/logos";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Avatar, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { API_URL, client } from "@/lib/api/client";
import { deleteAccount, getUser } from "@/lib/api/user";

export const Route = createFileRoute("/app/_appLayout/profile")({
  loader: async () => {
    const response = await client.GET("/auth/config");
    if (response.error) throw new Error("Failed to fetch auth configuration");
    return { authConfig: response.data };
  },
  component: ProfilePage,
});

function ProfilePage() {
  const { authConfig } = Route.useLoaderData();
  const queryClient = useQueryClient();
  const router = useRouter();
  const [deleteConfirmation, setDeleteConfirmation] = useState("");
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  const { data: user, isLoading } = useQuery({
    queryKey: ["user"],
    queryFn: getUser,
  });

  const deleteAccountMutation = useMutation({
    mutationFn: deleteAccount,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["user"] });
      await router.invalidate();
      queryClient.clear();
      queryClient.removeQueries();
      setIsDeleteDialogOpen(false);
    },
    onError: (error) => {
      console.error("Failed to delete account:", error);
    },
  });

  const handleDeleteAccount = () => {
    if (deleteConfirmation === "DELETE MY ACCOUNT") {
      deleteAccountMutation.mutate();
    }
  };

  const isDeleteConfirmationValid = deleteConfirmation === "DELETE MY ACCOUNT";

  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading profile...</p>
        </div>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  return (
    <div className="overflow-auto bg-background">
      <div className="container mx-auto px-4 py-8 max-w-3xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">Profile</h1>
          <p className="text-muted-foreground">Manage your account</p>
        </div>

        <div className="space-y-6">
          {/* User Information Card */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <User className="size-5" />
                Account Information
              </CardTitle>
              <CardDescription>Your account details</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-4">
                <Avatar>
                  <AvatarImage
                    src={user?.avatar_url || undefined}
                    alt="Avatar"
                  />
                </Avatar>
                <div>
                  <h3 className="font-semibold text-lg">{user.name}</h3>
                  <p className="text-sm text-muted-foreground">
                    User ID: {user.id}
                  </p>
                  {user.github_id && (
                    <p className="text-sm text-muted-foreground">
                      GitHub ID: {user.github_id}
                    </p>
                  )}
                  {user.google_id && (
                    <p className="text-sm text-muted-foreground">
                      Google ID: {user.google_id}
                    </p>
                  )}
                  {user.discord_id && (
                    <p className="text-sm text-muted-foreground">
                      Discord ID: {user.discord_id}
                    </p>
                  )}
                  {user.oidc_id && (
                    <p className="text-sm text-muted-foreground">
                      {authConfig.oidc?.name || "OIDC"} ID: {user.oidc_id}
                    </p>
                  )}
                  {user.sso_username && (
                    <p className="text-sm text-muted-foreground">
                      SSO Username: {user.sso_username}
                    </p>
                  )}
                </div>
              </div>
              <div className="flex gap-2">
                {!user.github_id && authConfig.github && (
                  <Button asChild variant="outline">
                    <a href={`${API_URL}/auth/login/github`}>
                      <GitHub />
                      Connect GitHub
                    </a>
                  </Button>
                )}
                {!user.google_id && authConfig.google && (
                  <Button asChild variant="outline">
                    <a href={`${API_URL}/auth/login/google`}>
                      <Google />
                      Connect Google
                    </a>
                  </Button>
                )}
                {!user.discord_id && authConfig.discord && (
                  <Button asChild variant="outline">
                    <a href={`${API_URL}/auth/login/discord`}>
                      <Discord />
                      Connect Discord
                    </a>
                  </Button>
                )}
                {!user.oidc_id && authConfig.oidc?.enabled && (
                  <Button asChild variant="outline">
                    <a href={`${API_URL}/auth/login/oidc`}>
                      <GlobeLock />
                      Connect {authConfig.oidc.name}
                    </a>
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>

          {/* Navigation Card */}
          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
              <CardDescription>
                Access other parts of your account
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              <Button
                asChild
                variant="outline"
                className="w-full justify-start"
              >
                <Link to="/app/providers">Manage Providers</Link>
              </Button>
              <Button
                asChild
                variant="outline"
                className="w-full justify-start"
              >
                <Link to="/app/api-keys">Manage API Keys</Link>
              </Button>
            </CardContent>
          </Card>

          {/* Danger Zone Card */}
          <Card className="border-destructive/50 dark:border-destructive/100">
            <CardHeader>
              <CardTitle className="text-destructive-foreground flex items-center gap-2">
                <AlertTriangle className="size-5" />
                Danger Zone
              </CardTitle>
              <CardDescription>
                Irreversible actions that will permanently affect your account
              </CardDescription>
            </CardHeader>
            <CardContent>
              <AlertDialog open={isDeleteDialogOpen}>
                <AlertDialogTrigger asChild>
                  <Button
                    variant="destructive"
                    className="w-full"
                    onClick={() => setIsDeleteDialogOpen(true)}
                  >
                    <Trash2 className="size-4 mr-2" />
                    Delete Account
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle className="text-destructive-foreground">
                      Delete Account Permanently
                    </AlertDialogTitle>
                    <AlertDialogDescription className="space-y-3">
                      This action cannot be undone. This will permanently delete
                      your account and remove all associated data.
                    </AlertDialogDescription>
                    <div className="text-sm">This will permanently delete:</div>
                    <ul className="list-disc list-inside space-y-1 text-sm">
                      <li>Your chat history and settings</li>
                      <li>Your stored API keys (encrypted)</li>
                      <li>Your account preferences</li>
                    </ul>
                    <p className="font-medium">
                      Type <strong>DELETE MY ACCOUNT</strong> below to confirm:
                    </p>
                    <div className="pt-2">
                      <input
                        type="text"
                        value={deleteConfirmation}
                        onChange={(e) => setDeleteConfirmation(e.target.value)}
                        placeholder="DELETE MY ACCOUNT"
                        className="w-full px-3 py-2 border border-input rounded-md bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                      />
                    </div>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel
                      onClick={() => {
                        setDeleteConfirmation("");
                        setIsDeleteDialogOpen(false);
                      }}
                    >
                      Cancel
                    </AlertDialogCancel>
                    <AlertDialogAction
                      variant="destructive"
                      onClick={handleDeleteAccount}
                      disabled={
                        !isDeleteConfirmationValid ||
                        deleteAccountMutation.isPending
                      }
                    >
                      {deleteAccountMutation.isPending ? (
                        <>
                          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                          Deleting...
                        </>
                      ) : (
                        "Delete Account"
                      )}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </CardContent>
          </Card>
        </div>

        <div className="mt-12 pt-8 border-t border-border">
          <div className="flex items-center justify-between text-sm text-muted-foreground">
            <div className="space-x-4">
              <a href="/legal/terms" className="hover:text-primary underline">
                Terms of Service
              </a>
              <a href="/legal/privacy" className="hover:text-primary underline">
                Privacy Policy
              </a>
            </div>
            <p>RsChat v1.0.0</p>
          </div>
        </div>
      </div>
    </div>
  );
}
