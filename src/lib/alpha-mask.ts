export function applyAlphaMask(
  imagePixels: Uint8ClampedArray,
  maskPixels: Uint8ClampedArray,
): void {
  if (maskPixels.length < imagePixels.length) {
    throw new RangeError("The AI mask dimensions do not match the source frame.");
  }
  for (let index = 0; index < imagePixels.length; index += 4) {
    const sourceAlpha = imagePixels[index + 3] ?? 0;
    const maskAlpha = maskPixels[index] ?? 0;
    imagePixels[index + 3] = Math.round((sourceAlpha * maskAlpha) / 255);
  }
}
