export interface IkihsToken {
  content: string;
  offset: number;
  color: string;
  fontStyle: string;
  scope: string;
  category: string;
}

export interface ShikiToken {
  content: string;
  offset: number;
  color: string;
  fontStyle: number;
}

export interface IkihsResult {
  tokens: IkihsToken[][];
  fg: string;
  bg: string;
  language: string;
}

export interface ShikiResult {
  tokens: ShikiToken[][];
  fg: string;
  bg: string;
  language: string;
}

export interface DiffToken {
  content: string;
  offset: number;
  ikihsColor: string;
  shikiColor: string;
  ikihsCategory: string;
  match: "exact" | "color_diff" | "missing_in_ikihs" | "missing_in_shiki" | "offset_diff" | "extra";
}
