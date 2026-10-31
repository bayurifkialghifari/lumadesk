import type { EditRecipe, MediaId } from "./contracts";

export function defaultEditRecipe(mediaId: MediaId): EditRecipe {
  return {
    mediaId,
    crop: null,
    rotationDegrees: 0,
    flipHorizontal: false,
    backgroundRemoval: "none",
    brightness: 1,
    contrast: 1,
    saturation: 1,
    updatedAt: new Date(0).toISOString(),
  };
}

export function editFilter(recipe: EditRecipe): string {
  return [
    `brightness(${recipe.brightness})`,
    `contrast(${recipe.contrast})`,
    `saturate(${recipe.saturation})`,
  ].join(" ");
}

export function editTransform(recipe: EditRecipe): string {
  const scaleX = recipe.flipHorizontal ? -1 : 1;
  return `rotate(${recipe.rotationDegrees}deg) scaleX(${scaleX})`;
}

export function recipeIsDefault(recipe: EditRecipe): boolean {
  return (
    recipe.crop === null &&
    recipe.rotationDegrees === 0 &&
    !recipe.flipHorizontal &&
    recipe.backgroundRemoval === "none" &&
    recipe.brightness === 1 &&
    recipe.contrast === 1 &&
    recipe.saturation === 1
  );
}
