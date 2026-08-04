import { createApp, defineComponent, type Plugin } from "vue"

export function withSetup<T>(composable: () => T, plugins: Plugin[] = []): [T, () => void] {
  let result!: T
  const app = createApp(
    defineComponent({
      setup() { result = composable(); return () => {} },
      template: "<div />",
    }),
  )
  plugins.forEach((p) => app.use(p))
  app.mount(document.createElement("div"))
  return [result, () => app.unmount()]
}
