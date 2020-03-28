import * as vscode from 'vscode';

interface CargoPlayTaskDefinition extends vscode.TaskDefinition {
	/* The Rust file to compile */
	target_file: string;
}

export async function activate(context: vscode.ExtensionContext) {
	let play = vscode.commands.registerCommand('extension.cargoPlay', async () => {
		const filename = vscode.window.activeTextEditor?.document.fileName;
		if (filename === undefined || !filename.endsWith(".rs")) {
			vscode.window.showWarningMessage("CargoPlay: No active Rust file found");
			return;
		}

		let def: CargoPlayTaskDefinition = {
			type: 'cargo-play',
			target_file: filename
		};
		let execution = new vscode.ShellExecution(`cargo play ${def.target_file}`);
		let task = new vscode.Task(def, vscode.TaskScope.Global.toString(), 'cargo-play', execution, ["$rustc"]);
		vscode.tasks.executeTask(task);
	});
	context.subscriptions.push(play);

	let install = vscode.commands.registerCommand('extension.installCargoPlay', async () => {
		let def: vscode.TaskDefinition = { type: 'cargo-play-install' };
		let execution = new vscode.ShellExecution(`cargo install cargo-play`);
		let task = new vscode.Task(def, vscode.TaskScope.Global.toString(), 'cargo-play-install', execution, ["$rustc"]);
		await vscode.tasks.executeTask(task);
	});
	context.subscriptions.push(install);
}

export function deactivate() { }
