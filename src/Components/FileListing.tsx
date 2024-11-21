import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FileRow from "./FileRow";

function FileListing({path, files}) {
    return (
    <table>
        <thead>
            <tr>
                <th>Files found in path {path}</th>
            </tr>
        </thead>
        <tbody>
            {files.map((e: String) => <FileRow file={e}></FileRow>)}
        </tbody>
    </table>
    );
}

export default FileListing;