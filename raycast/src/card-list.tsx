import { Action, ActionPanel, Alert, Form, Icon, List, Toast, confirmAlert, showToast, useNavigation } from "@raycast/api";
import { useState } from "react";
import { Hand, copyCard, removeCard, removeHand, renameCardKey, upsertCard } from "./utils/hc";

export function CardList({ hand, onHandChange }: { hand: Hand; onHandChange: () => void }) {
  const [cards, setCards] = useState<string[]>(hand.cards);
  const { pop } = useNavigation();

  async function handleRemoveHand() {
    const confirmed = await confirmAlert({
      title: `Delete '${hand.name}'?`,
      message: "This action cannot be undone.",
      primaryAction: { title: "Delete", style: Alert.ActionStyle.Destructive },
    });
    if (!confirmed) return;

    const toast = await showToast({ style: Toast.Style.Animated, title: "Deleting..." });
    try {
      removeHand(hand.name);
      toast.style = Toast.Style.Success;
      toast.title = `Hand '${hand.name}' deleted`;
      onHandChange();
      pop();
    } catch (e) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to delete hand";
      toast.message = e instanceof Error ? e.message : String(e);
    }
  }

  function handleCardRemoved(key: string) {
    setCards((prev) => prev.filter((c) => c !== key));
    onHandChange();
  }

  function handleCardAdded(key: string) {
    setCards((prev) => (prev.includes(key) ? prev : [...prev, key]));
    onHandChange();
  }

  function handleCardRenamed(oldKey: string, newKey: string) {
    setCards((prev) => prev.map((c) => (c === oldKey ? newKey : c)));
    onHandChange();
  }

  return (
    <List
      navigationTitle={hand.name}
      searchBarPlaceholder="Search cards..."
      actions={
        <ActionPanel>
          <Action.Push
            title="Add Card"
            icon={Icon.Plus}
            target={
              <UpsertCardForm
                handName={hand.name}
                onComplete={(key) => handleCardAdded(key)}
              />
            }
          />
          <Action
            title="Delete Hand"
            icon={Icon.Trash}
            style={Action.Style.Destructive}
            onAction={handleRemoveHand}
          />
        </ActionPanel>
      }
    >
      {cards.map((card) => (
        <CardItem
          key={card}
          handName={hand.name}
          card={card}
          onRemoved={() => handleCardRemoved(card)}
          onRenamed={(newKey) => handleCardRenamed(card, newKey)}
          onCardAdded={handleCardAdded}
        />
      ))}
    </List>
  );
}

function CardItem({
  handName,
  card,
  onRemoved,
  onRenamed,
  onCardAdded,
}: {
  handName: string;
  card: string;
  onRemoved: () => void;
  onRenamed: (newKey: string) => void;
  onCardAdded: (key: string) => void;
}) {
  async function handleRemoveCard() {
    const confirmed = await confirmAlert({
      title: `Remove card '${card}'?`,
      primaryAction: { title: "Remove", style: Alert.ActionStyle.Destructive },
    });
    if (!confirmed) return;

    const toast = await showToast({ style: Toast.Style.Animated, title: "Removing..." });
    try {
      removeCard(handName, card);
      toast.style = Toast.Style.Success;
      toast.title = `Card '${card}' removed`;
      onRemoved();
    } catch (e) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to remove card";
      toast.message = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCopy() {
    const toast = await showToast({ style: Toast.Style.Animated, title: "Copying..." });
    try {
      copyCard(handName, card);
      toast.style = Toast.Style.Success;
      toast.title = `Copied ${card}`;
      toast.message = "Cleared from clipboard in 30s";
    } catch (e) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to copy";
      toast.message = e instanceof Error ? e.message : String(e);
    }
  }

  return (
    <List.Item
      icon={card === "password" ? Icon.Lock : Icon.Tag}
      title={card}
      accessories={[{ text: "••••••••" }]}
      actions={
        <ActionPanel>
          <Action title={`Copy ${card}`} icon={Icon.Clipboard} onAction={handleCopy} />
          <Action.Push
            title="Edit Value"
            icon={Icon.Pencil}
            target={<UpsertCardForm handName={handName} existingKey={card} onComplete={() => {}} />}
          />
          <Action.Push
            title="Rename Key"
            icon={Icon.TextCursor}
            target={<RenameCardKeyForm handName={handName} existingKey={card} onComplete={onRenamed} />}
          />
          <Action.Push
            title="Add Card"
            icon={Icon.Plus}
            shortcut={{ modifiers: ["cmd"], key: "n" }}
            target={<UpsertCardForm handName={handName} onComplete={onCardAdded} />}
          />
          <Action
            title="Remove Card"
            icon={Icon.Trash}
            style={Action.Style.Destructive}
            shortcut={{ modifiers: ["ctrl"], key: "x" }}
            onAction={handleRemoveCard}
          />
        </ActionPanel>
      }
    />
  );
}

function UpsertCardForm({
  handName,
  existingKey,
  onComplete,
}: {
  handName: string;
  existingKey?: string;
  onComplete: (key: string) => void;
}) {
  const { pop } = useNavigation();
  const isEdit = !!existingKey;

  async function handleSubmit(values: { key: string; value: string }) {
    const key = existingKey ?? values.key.trim();
    if (!key) {
      await showToast({ style: Toast.Style.Failure, title: "Key is required" });
      return;
    }
    if (!values.value.trim()) {
      await showToast({ style: Toast.Style.Failure, title: "Value is required" });
      return;
    }

    const toast = await showToast({ style: Toast.Style.Animated, title: isEdit ? "Updating..." : "Adding..." });
    try {
      upsertCard(handName, key, values.value);
      toast.style = Toast.Style.Success;
      toast.title = isEdit ? `Card '${key}' updated` : `Card '${key}' added`;
      onComplete(key);
      pop();
    } catch (e) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed";
      toast.message = e instanceof Error ? e.message : String(e);
    }
  }

  return (
    <Form
      navigationTitle={isEdit ? `Edit '${existingKey}'` : "Add Card"}
      actions={
        <ActionPanel>
          <Action.SubmitForm title={isEdit ? "Update" : "Add Card"} onSubmit={handleSubmit} />
        </ActionPanel>
      }
    >
      {!isEdit && <Form.TextField id="key" title="Key" placeholder="username" />}
      <Form.PasswordField id="value" title="Value" placeholder={isEdit ? "New value" : "Value"} />
    </Form>
  );
}

function RenameCardKeyForm({
  handName,
  existingKey,
  onComplete,
}: {
  handName: string;
  existingKey: string;
  onComplete: (newKey: string) => void;
}) {
  const { pop } = useNavigation();

  async function handleSubmit(values: { newKey: string }) {
    const newKey = values.newKey.trim();
    if (!newKey) {
      await showToast({ style: Toast.Style.Failure, title: "New key name is required" });
      return;
    }
    if (newKey === existingKey) {
      await showToast({ style: Toast.Style.Failure, title: "New key is the same as current" });
      return;
    }

    const toast = await showToast({ style: Toast.Style.Animated, title: "Renaming..." });
    try {
      renameCardKey(handName, existingKey, newKey);
      toast.style = Toast.Style.Success;
      toast.title = `Card renamed '${existingKey}' → '${newKey}'`;
      onComplete(newKey);
      pop();
    } catch (e) {
      toast.style = Toast.Style.Failure;
      toast.title = "Failed to rename card";
      toast.message = e instanceof Error ? e.message : String(e);
    }
  }

  return (
    <Form
      navigationTitle={`Rename '${existingKey}'`}
      actions={
        <ActionPanel>
          <Action.SubmitForm title="Rename" onSubmit={handleSubmit} />
        </ActionPanel>
      }
    >
      <Form.TextField id="newKey" title="New Key Name" defaultValue={existingKey} />
    </Form>
  );
}
