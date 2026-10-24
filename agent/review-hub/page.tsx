'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"

export default function Home() {
  return (
    <div className="min-h-screen flex flex-col bg-background text-foreground">
      <main className="flex-1 mx-auto w-full max-w-4xl px-4 py-10 sm:px-6">
        <header className="mb-8">
          <h1 className="text-3xl font-bold tracking-tight">Photocraft Review Hub</h1>
          <p className="mt-2 text-muted-foreground">
            The locally built photocraft web app (Rust + egui + WebGPU) and its
            annotation-equipped review surface, baked static and served from this
            one server.
          </p>
        </header>

        <div className="grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="text-xl">The App</CardTitle>
              <CardDescription>
                The full editor, running in your browser as WebAssembly.
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-4">
              <p className="text-sm text-muted-foreground">
                Fresh build of <code className="text-xs">feature/right-panel-collapse</code>:
                the icon rail, the direct dock toggle, the collapsed-dock peek
                flyouts, and the slide-out dock column.
              </p>
              <Button asChild className="w-fit">
                <a href="/photocraft">Open /photocraft</a>
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-xl">The Review Surface</CardTitle>
              <CardDescription>
                The app embedded in Storybook with the Annotakit pin layer.
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-4">
              <p className="text-sm text-muted-foreground">
                Pin comments and region notes on the live UI; threads live in
                your browser and export to markdown/JSON for hand-carrying.
              </p>
              <Button asChild className="w-fit">
                <a href="/storybook">Open /storybook</a>
              </Button>
            </CardContent>
          </Card>
        </div>

        <Separator className="my-10" />

        <section>
          <h2 className="text-lg font-semibold mb-3">How to review</h2>
          <ul className="space-y-2 text-sm text-muted-foreground list-disc pl-5">
            <li>
              <Badge variant="secondary" className="mr-2">pin</Badge>
              Click any element to start a numbered thread. Against the canvas
              app, pins anchor to the embed; use <Badge variant="secondary" className="mx-1">region</Badge>
              for canvas areas (toolbar, panels, canvas chrome).
            </li>
            <li>
              <Badge variant="secondary" className="mr-2">freeze</Badge>
              Captures hover-driven UI so flyout menus stay still while you pin.
            </li>
            <li>
              Hotkeys: <code className="text-xs">⌥C</code> pin ·{" "}
              <code className="text-xs">⌥B</code> region ·{" "}
              <code className="text-xs">⌥I</code> freeze ·{" "}
              <code className="text-xs">⌥P</code> threads panel.
            </li>
            <li>
              Threads are stored in this browser. Use the panel&apos;s{" "}
              <em>copy md</em> / <em>download json</em> exports to hand feedback
              to the agent until the GitHub mirror is armed with a scoped token.
            </li>
          </ul>
        </section>
      </main>

      <footer className="mt-auto border-t bg-muted/40">
        <div className="mx-auto w-full max-w-4xl px-4 py-4 text-xs text-muted-foreground sm:px-6">
          photocraft review hub · static bakes regenerate from the work repo&apos;s
          agent scripts after each build
        </div>
      </footer>
    </div>
  )
}
