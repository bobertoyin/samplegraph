// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { SongInfo } from "./SongInfo.d";

export interface GraphResponse {
    graph: { nodes: Array<number>; edges: Array<[number, number, string]> };
    songs: Record<number, SongInfo>;
}
