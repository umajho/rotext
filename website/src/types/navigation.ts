export interface Navigation {
  isPlain?: true;
  name: string;
  realName?: string;
  children?: Navigation[];
}
