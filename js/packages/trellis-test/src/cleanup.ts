import { join } from "@std/path";

let hasProc: boolean | undefined;

async function canCheckProcesses(): Promise<boolean> {
  if (hasProc !== undefined) return hasProc;
  try {
    const stat = await Deno.stat("/proc");
    hasProc = stat.isDirectory;
  } catch {
    hasProc = false;
  }
  return hasProc;
}

async function processIsGone(pid: number): Promise<boolean> {
  if (!await canCheckProcesses()) return false;
  try {
    await Deno.stat(`/proc/${pid}`);
    return false;
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return true;
    return false;
  }
}

function parsePid(value: string): number | undefined {
  const pid = Number(value.trim());
  return Number.isSafeInteger(pid) && pid > 0 ? pid : undefined;
}

/** @internal Writes the owning process marker used to reap abandoned test directories. */
export async function writeTrellisTestOwnerMarker(
  dir: string,
  markerName: string,
): Promise<void> {
  await Deno.writeTextFile(join(dir, markerName), `${Deno.pid}\n`);
}

/** @internal Removes marked test directories whose owner process is gone. */
export async function removeStaleMarkedDirectories(args: {
  readonly parent: string;
  readonly prefix: string;
  readonly markerName: string;
}): Promise<void> {
  let entries: Deno.DirEntry[] = [];
  try {
    for await (const entry of Deno.readDir(args.parent)) entries.push(entry);
  } catch {
    return;
  }

  for (const entry of entries) {
    if (!entry.isDirectory || !entry.name.startsWith(args.prefix)) continue;
    const path = join(args.parent, entry.name);
    const marker = await Deno.readTextFile(join(path, args.markerName)).catch(
      () => undefined,
    );
    const pid = marker === undefined ? undefined : parsePid(marker);
    if (pid !== undefined && await processIsGone(pid)) {
      await Deno.remove(path, { recursive: true }).catch(() => undefined);
    }
  }
}

/** @internal Removes PID-named resources whose owner process is gone. */
export async function removeStalePidNamedResources(args: {
  readonly names: readonly string[];
  readonly prefix: string;
  readonly remove: (name: string) => Promise<void>;
}): Promise<void> {
  for (const name of args.names) {
    if (!name.startsWith(args.prefix)) continue;
    const pid = parsePid(name.slice(args.prefix.length).split("-", 1)[0]);
    if (pid !== undefined && await processIsGone(pid)) await args.remove(name);
  }
}
