import { showHUD } from "@raycast/api";
import { triggerAction } from "./utils";

export default async function Command() {
  await triggerAction("capture-screen");
  await showHUD("Capture screen triggered");
}
