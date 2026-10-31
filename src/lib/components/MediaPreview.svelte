<script lang="ts">
import type { EditRecipe, MediaItem } from "../contracts";
import { defaultEditRecipe } from "../edit-recipes";
import EditedMediaPreview from "./EditedMediaPreview.svelte";

export let item: MediaItem;
export let src: string | undefined = undefined;
export let recipe: EditRecipe | undefined = undefined;
export let eager = false;

$: activeRecipe = recipe ?? defaultEditRecipe(item.id);
</script>

<EditedMediaPreview {item} {src} recipe={activeRecipe} {eager} />
