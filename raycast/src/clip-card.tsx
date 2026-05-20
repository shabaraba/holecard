import { Action, ActionPanel, Icon, List, Toast, showToast } from "@raycast/api";
import { useCallback, useEffect, useState } from "react";
import { AddHandForm } from "./manage-hand";
import { CardList } from "./card-list";
import { LockedView } from "./components/locked-view";
import { Hand, copyCard, listHands } from "./utils/hc";

export default function SearchHands() {
  const [hands, setHands] = useState<Hand[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    setIsLoading(true);
    try {
      setHands(listHands());
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  if (error) {
    return <LockedView description={error} />;
  }

  return (
    <List
      isLoading={isLoading}
      searchBarPlaceholder="Search hands..."
      actions={
        <ActionPanel>
          <Action.Push title="Add Hand" icon={Icon.Plus} target={<AddHandForm onComplete={load} />} />
        </ActionPanel>
      }
    >
      {hands.map((hand) => (
        <HandItem key={hand.name} hand={hand} onHandChange={load} />
      ))}
    </List>
  );
}

function HandItem({ hand, onHandChange }: { hand: Hand; onHandChange: () => void }) {
  const primaryCard = hand.cards.includes("password") ? "password" : hand.cards[0];

  return (
    <List.Item
      icon={Icon.Key}
      title={hand.name}
      subtitle={hand.cards.join(", ")}
      accessories={[{ text: `${hand.cards.length} cards` }]}
      actions={
        <ActionPanel>
          <Action.Push
            title="Manage Cards"
            icon={Icon.ChevronRight}
            target={<CardList hand={hand} onHandChange={onHandChange} />}
          />
          {primaryCard && (
            <Action
              title={`Copy ${primaryCard}`}
              icon={Icon.Clipboard}
              shortcut={{ modifiers: ["cmd"], key: "c" }}
              onAction={() => handleCopy(hand.name, primaryCard)}
            />
          )}
          <Action.Push
            title="Add Hand"
            icon={Icon.Plus}
            shortcut={{ modifiers: ["cmd"], key: "n" }}
            target={<AddHandForm onComplete={onHandChange} />}
          />
        </ActionPanel>
      }
    />
  );
}

async function handleCopy(handName: string, cardName: string) {
  const toast = await showToast({ style: Toast.Style.Animated, title: "Copying..." });
  try {
    copyCard(handName, cardName);
    toast.style = Toast.Style.Success;
    toast.title = `Copied ${cardName}`;
    toast.message = "Cleared from clipboard in 30s";
  } catch (e) {
    toast.style = Toast.Style.Failure;
    toast.title = "Failed to copy";
    toast.message = e instanceof Error ? e.message : String(e);
  }
}
