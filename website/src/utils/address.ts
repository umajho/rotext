export type Address =
  | [
    type: "reference/textualAbsolute",
    prefix: string,
    /**
     * 可以是主串（如 “abc”）也可以是子串（如 “abc.def”）。
     */
    threadID: string,
    floorNumber: number | null,
  ]
  | [
    type: "reference/textualFloorNumber",
    floorNumber: number,
  ]
  | [
    type: "reference/numeric",
    prefix: string,
    id: number,
  ]
  | [type: "wiki", fullName: string | null, anchor: string | null]
  | [type: "live"]
  | [type: "never"];

export function stringifyAddress(addr: Address): string {
  return JSON.stringify(addr);
}

export function reconstructAddressAsText(
  address: Extract<
    Address,
    {
      0:
        | "reference/textualAbsolute"
        | "reference/textualFloorNumber"
        | "reference/numeric"
        | "wiki";
    }
  >,
): string {
  switch (address[0]) {
    case "reference/textualAbsolute": {
      const [_, prefix, threadID, floorNumber] = address;
      return `>>${prefix}.${threadID}` +
        (floorNumber !== null ? `#${floorNumber}` : "");
    }
    case "reference/textualFloorNumber": {
      const [_, floorNumber] = address;
      return `>>#${floorNumber}`;
    }
    case "reference/numeric": {
      const [_, prefix, id] = address;
      return `>>${prefix}.${id}`;
    }
    case "wiki":
      const [_, fullName, anchor] = address;
      return `[[${fullName}${anchor !== null ? `#${anchor}` : ""}]]`;
    default:
      address satisfies never;
      throw new Error("unreachable");
  }
}
