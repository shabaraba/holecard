import { Action, ActionPanel, Alert, Form, Icon, List, Toast, confirmAlert, showToast, useNavigation } from "@raycast/api";
import { Hand, copyCard, removeCard, removeHand, upsertCard } from "./utils/hc";

export function CardList({ hand, onHandChange }: { hand: Hand; onHandChange: () => void }) {
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

  return (
    <List
      navigationTitle={hand.name}
      searchBarPlaceholder="Search cards..."
      actions={
        <ActionPanel>
          <Action.Push
            title="Add Card"
            icon={Icon.Plus}
            target={<UpsertCardForm handName={hand.name} onComplete={onHandChange} />}
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
      {hand.cards.map((card) => (
        <CardItem key={card} handName={hand.name} card={card} onHandChange={onHandChange} />
      ))}
    </List>
  );
}

function CardItem({
  handName,
  card,
  onHandChange,
}: {
  handName: string;
  card: string;
  onHandChange: () => void;
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
      onHandChange();
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
            target={<UpsertCardForm handName={handName} existingKey={card} onComplete={onHandChange} />}
          />
          <Action.Push
            title="Add Card"
            icon={Icon.Plus}
            target={<UpsertCardForm handName={handName} onComplete={onHandChange} />}
          />
          <Action
            title="Remove Card"
            icon={Icon.Trash}
            style={Action.Style.Destructive}
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
  onComplete: () => void;
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
      onComplete();
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
