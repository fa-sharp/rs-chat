import { Link } from "@tanstack/react-router";
import { GlobeLock } from "lucide-react";

import { Discord, GitHub, Google } from "@/components/logos";
import { Button } from "@/components/ui/button";
import { API_URL } from "@/lib/api/client";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  config: components["schemas"]["AuthConfig"];
}

export function LoginForm({
  className,
  config,
  ...props
}: React.ComponentProps<"div"> & Props) {
  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <form>
        <div className="flex flex-col gap-6">
          <div className="flex flex-col items-center gap-2">
            <div className="flex flex-col items-center gap-2 font-medium">
              <img src="/logo512.png" alt="RsChat Logo" className="size-20" />
              <span className="sr-only">RsChat</span>
            </div>
            <h1 className="text-2xl font-bold">Welcome to RsChat</h1>
            <div className="text-center text-sm">
              A submission to the{" "}
              <a
                target="_blank"
                rel="noopener noreferrer"
                href="https://cloneathon.t3.chat/"
                className="underline underline-offset-4"
              >
                T3 Chat Cloneathon
              </a>
            </div>
          </div>
          <div className="flex flex-col gap-2">
            {config.github && (
              <Button asChild variant="outline" type="button">
                <a href={`${API_URL}/auth/login/github`}>
                  <GitHub />
                  Login with GitHub
                </a>
              </Button>
            )}
            {config.google && (
              <Button
                asChild
                variant="outline"
                type="button"
                className="w-full"
              >
                <a href={`${API_URL}/auth/login/google`}>
                  <Google />
                  Login with Google
                </a>
              </Button>
            )}
            {config.discord && (
              <Button
                asChild
                variant="outline"
                type="button"
                className="w-full"
              >
                <a href={`${API_URL}/auth/login/discord`}>
                  <Discord />
                  Login with Discord
                </a>
              </Button>
            )}
            {config.oidc?.enabled && (
              <Button
                asChild
                variant="outline"
                type="button"
                className="w-full"
              >
                <a href={`${API_URL}/auth/login/oidc`}>
                  <GlobeLock />
                  Login with {config.oidc.name}
                </a>
              </Button>
            )}
          </div>
        </div>
      </form>
      <div className="text-muted-foreground *:[a]:hover:text-primary text-center text-xs text-balance *:[a]:underline *:[a]:underline-offset-4">
        By continuing, you agree to our{" "}
        <Link to="/legal/terms">Terms of Service</Link> and{" "}
        <Link to="/legal/privacy">Privacy Policy</Link>.
      </div>
    </div>
  );
}
