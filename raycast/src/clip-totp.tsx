import { Action, ActionPanel, Clipboard, Color, Icon, List, Toast, showToast } from "@raycast/api";
import { useEffect, useState } from "react";
import { LockedView } from "./components/locked-view";
import { TotpEntry, TotpResult, fetchTotpEntry, listTotpServices } from "./utils/hc";

export default function SearchTotp() {
  const [entries, setEntries] = useState<TotpEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    try {
      const services = listTotpServices();
      setEntries(services.map(fetchTotpEntry));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setIsLoading(false);
      return;
    }
    setIsLoading(false);

    const timer = setInterval(() => {
      setEntries((prev) =>
        prev.map((entry) => {
          if (!entry.result) return entry;
          const next = entry.result.remainingSeconds - 1;
          if (next <= 0) return fetchTotpEntry(entry.service);
          return { ...entry, result: { ...entry.result, remainingSeconds: next } };
        }),
      );
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  if (error) {
    return <LockedView description={error} />;
  }

  return (
    <List isLoading={isLoading} searchBarPlaceholder="Search TOTP services...">
      {entries.map((entry) => (
        <TotpItem key={entry.service} entry={entry} />
      ))}
    </List>
  );
}

function TotpItem({ entry }: { entry: TotpEntry }) {
  const { service, result, error } = entry;

  const accessories: List.Item.Accessory[] = result
    ? [
        {
          tag: {
            value: `${result.remainingSeconds}s`,
            color: result.remainingSeconds <= 5 ? Color.Red : result.remainingSeconds <= 10 ? Color.Orange : Color.Green,
          },
        },
      ]
    : [];

  return (
    <List.Item
      icon={Icon.Clock}
      title={service}
      subtitle={result ? result.code : error ?? "Loading..."}
      accessories={accessories}
      actions={
        <ActionPanel>
          <Action
            title="Copy TOTP Code"
            icon={Icon.Clipboard}
            onAction={() => handleCopyTotp(service)}
          />
        </ActionPanel>
      }
    />
  );
}

async function handleCopyTotp(service: string) {
  const toast = await showToast({ style: Toast.Style.Animated, title: "Copying TOTP..." });
  try {
    const latest = fetchTotpEntry(service).result!;
    await Clipboard.copy(latest.code);
    toast.style = Toast.Style.Success;
    toast.title = `Copied TOTP for ${service}`;
    toast.message = `Valid for ${latest.remainingSeconds}s`;
  } catch (e) {
    toast.style = Toast.Style.Failure;
    toast.title = "Failed to get TOTP";
    toast.message = e instanceof Error ? e.message : String(e);
  }
}
