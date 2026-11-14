import { useActionState } from "react";
import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";

import { fetchMetadata, login } from "@/api";
import { Logo } from "@/components/logo";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export const Route = createFileRoute("/login")({
  beforeLoad: async () => {
    const allowed = await fetchMetadata().then(
      () => true,
      () => false,
    );
    if (allowed) throw redirect({ to: "/" });
  },
  component: Login,
});

function Login() {
  const navigate = useNavigate();
  const [failed, submit, pending] = useActionState(
    async (_: boolean, form: FormData) => {
      const res = await login(
        String(form.get("username")),
        String(form.get("password")),
      );
      if (res.ok) await navigate({ to: "/" });
      return !res.ok;
    },
    false,
  );

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-8 p-4">
      <Logo className="w-48" />
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle className="text-primary">Log in</CardTitle>
        </CardHeader>
        <CardContent>
          <form action={submit} className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                name="username"
                autoComplete="username"
                autoFocus
                required
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                required
              />
            </div>
            {failed && (
              <p className="text-sm text-destructive">
                Wrong username or password.
              </p>
            )}
            <Button type="submit" disabled={pending}>
              Log in
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
