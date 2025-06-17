import { Github } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Link } from "@tanstack/react-router";

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <form>
        <div className="flex flex-col gap-6">
          <div className="flex flex-col items-center gap-2">
            <div className="flex flex-col items-center gap-2 font-medium">
              <img src="/logo512.png" alt="RsChat Logo" className="size-20" />
              <span className="sr-only">RsChat</span>
            </div>
            <h1 className="text-xl font-bold">Welcome to RsChat</h1>
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
          {/* <div className="flex flex-col gap-6">
            <div className="grid gap-3">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="m@example.com"
                required
              />
            </div>
            <Button type="submit" className="w-full">
              Login
            </Button>
          </div>
          <div className="after:border-border relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t">
            <span className="bg-background text-muted-foreground relative z-10 px-2">
              Or
            </span>
          </div> */}
          <div className="flex">
            <Button asChild variant="outline" type="button" className="w-full">
              <a href="/api/oauth/login/github">
                <Github className="size-5" />
                Login with GitHub
              </a>
            </Button>
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
