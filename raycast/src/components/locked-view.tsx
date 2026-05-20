import { Color, Icon, List } from "@raycast/api";

export function LockedView({ description }: { description: string }) {
  return (
    <List>
      <List.EmptyView
        icon={{ source: Icon.ExclamationMark, tintColor: Color.Red }}
        title="Holecard Locked"
        description={description}
      />
    </List>
  );
}
