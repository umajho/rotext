export type Address =
  | [
    type: "reference/textual",
    prefix: string,
    /**
     * 可以是主串（如 “abc”）也可以是子串（如 “abc.def”）。
     */
    threadID: string,
    floorNumber: number | null,
  ]
  | [
    type: "reference/numeric",
    prefix: string,
    id: number,
  ]
  | [type: "internal", fullName: string | null, anchor: string | null]
  | [type: "live"]
  | [type: "never"];

export function stringifyAddress(addr: Address): string {
  return JSON.stringify(addr);
}

export function reconstructAddressAsText(
  address: Extract<
    Address,
    { 0: "reference/textual" | "reference/numeric" | "internal" }
  >,
): string {
  switch (address[0]) {
    case "reference/textual": {
      const [_, prefix, threadID, floorNumber] = address;
      return `>>${prefix}.${threadID}` +
        (floorNumber !== null ? `#${floorNumber}` : "");
    }
    case "reference/numeric": {
      const [_, prefix, id] = address;
      return `>>${prefix}.${id}`;
    }
    case "internal":
      const [_, fullName, anchor] = address;
      return `[[${fullName}${anchor !== null ? `#${anchor}` : ""}]]`;
    default:
      address satisfies never;
      throw new Error("unreachable");
  }
}
