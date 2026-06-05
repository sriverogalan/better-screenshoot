import { showHUD } from "@raycast/api";
import { triggerAction } from "./utils";

export default async function Command() {
  await triggerAction("capture-area");
  await showHUD("Capture area triggered");
}
