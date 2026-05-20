import { Action, ActionPanel, Form, Icon, List, Toast, showToast, useNavigation } from "@raycast/api";
import { useCallback, useEffect, useState } from "react";
import { CardList } from "./card-list";
import { LockedView } from "./components/locked-view";
import { Hand, addHand, listHands } from "./utils/hc";

export default function ManageHand() {
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
        <List.Item
          key={hand.name}
          icon={Icon.Key}
          title={hand.name}
          subtitle={hand.cards.join(", ")}
          accessories={[{ text: `${hand.cards.length} cards` }]}
          actions={
            <ActionPanel>
              <Action.Push
                title="Manage Cards"
                icon={Icon.ChevronRight}
                target={<CardList hand={hand} onHandChange={load} />}
              />
              <Action.Push
                title="Add Hand"
                icon={Icon.Plus}
                shortcut={{ modifiers: ["cmd"], key: "n" }}
                target={<AddHandForm onComplete={load} />}
              />
            </ActionPanel>
          }
        />
      ))}
    </List>
  );
}

interface FormValues {
  name: string;
  note: string;
  key1: string; val1: string;
  key2: string; val2: string;
  key3: string; val3: string;
  key4: string; val4: string;
}

export function AddHandForm({ onComplete }: { onComplete?: () => void }) {
  const { pop } = useNavigation();

  async function handleSubmit(values: FormValues) {
    if (!values.name.trim()) {
      await showToast({ style: Toast.Style.Failure, title: "Hand name is required" });
      return;
    }

    const cards: Record<string, string> = {};
    for (let i = 1; i <= 4; i++) {
      const k = values[`key${i}` as keyof FormValues].trim();
      const v = values[`val${i}` as keyof FormValues].trim();
      if (k && v) cards[k] = v;
    }

    const toast = await showToast({ style: Toast.Style.Animated, title: "Adding hand..." });
    try {
      addHand(values.name.trim(), cards, values.note);
      toast.style = Toast.Style.Success;
      toast.title = `Hand '${values.name}' added`;
      onComplete?.();
      pop();
    } catch (e) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to add hand";
      toast.message = e instanceof Error ? e.message : String(e);
    }
  }

  return (
    <Form
      navigationTitle="Add Hand"
      actions={
        <ActionPanel>
          <Action.SubmitForm title="Add Hand" onSubmit={handleSubmit} />
        </ActionPanel>
      }
    >
      <Form.TextField id="name" title="Hand Name" placeholder="github" />
      <Form.Separator />
      <Form.TextField id="key1" title="Card 1 Key" placeholder="username" />
      <Form.PasswordField id="val1" title="Card 1 Value" />
      <Form.TextField id="key2" title="Card 2 Key" placeholder="password" />
      <Form.PasswordField id="val2" title="Card 2 Value" />
      <Form.TextField id="key3" title="Card 3 Key" placeholder="token" />
      <Form.PasswordField id="val3" title="Card 3 Value" />
      <Form.TextField id="key4" title="Card 4 Key" />
      <Form.PasswordField id="val4" title="Card 4 Value" />
      <Form.Separator />
      <Form.TextField id="note" title="Notes" placeholder="Optional notes" />
    </Form>
  );
}
