/// <reference lib="webworker" />

import * as Comlink from "comlink";

import Airmail, { get_dataset_info, search } from "../pkg";
import { files } from "./fetch_directory";
import { DatasetInfo, Stat } from "./types";

var has_init = false;

async function maybeInit() {
  if (!has_init) {
    console.log(await Airmail());
    has_init = true;
  }
}

console.log("S", location.search);
function getReadStats(): Stat[] {
  const readByReason = new Map<string, Stat>();
  for (const [name, file] of files) {
    for (const read of file.readPages) {
      for (const reason of [read.reason, "Total"]) {
        const readData = file.chunkSize * (read.prefetch + 1);
        let t = readByReason.get(reason);
        if (!t) {
          t = {
            reason: reason,
            fetchedAmount: 0,
            requestCount: 0,
            totalReadCount: 0,
            cachedReadAmount: 0,
          };
          readByReason.set(reason, t);
        }
        if (!read.wasCached) {
          t.fetchedAmount += readData;
          t.requestCount += 1;
        } else {
          t.cachedReadAmount += readData;
        }
        t.totalReadCount += 1;
      }
    }
  }
  const arr = [...readByReason.values()];
  arr.sort((a, b) => b.fetchedAmount - a.fetchedAmount);
  return arr;
}

type SearchParams = {
  indexUrl: string;
  fields?: string[];
  rank: boolean;
  searchText: string;
  chunkSize: number;
};
export type Progress = {
  inc: number,
  message?: string
}
export function tantivyLog(log: string) {
  progressCallback({ inc: 0, message: log })
}
export function progressCallback(p: Progress) {
  self.postMessage({ type: "progress", data: p });
}
const api = {
  async search(data: SearchParams) {
    console.log("Searching")
    await maybeInit();
    for (const file of files.values()) {
      file.readPages = [];
    }
    return JSON.parse(
      search(data.indexUrl, data.chunkSize, data.fields, data.rank, data.searchText)
    );
  },
  getReadStats,
  async getIndexStats(indexUrl: string, chunkSize: number): Promise<DatasetInfo> {
    console.log("getting dataset info");
    await maybeInit();
    return JSON.parse(get_dataset_info(indexUrl, chunkSize));
  },
  dumpCache() {
    const fs = [];
    for (const [name, f] of files) {
      fs.push([name, f.getCachedChunks()] as const)
    }
    return fs;
  }
};
export type Api = typeof api;
Comlink.expose(api);

self.postMessage("inited");
