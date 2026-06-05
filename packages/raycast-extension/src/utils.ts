import { execFile } from "child_process";
import { promisify } from "util";

const execFileAsync = promisify(execFile);

export async function triggerAction(action: string): Promise<void> {
  const url = `betterscreenshoot://${action}`;
  await execFileAsync("open", [url]);
}
