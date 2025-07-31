import { predict } from "./pkg/sonai";
import "./style.css";

const $input = document.getElementById("input") as HTMLTextAreaElement;
const $output = document.getElementById("output") as HTMLPreElement;

$input.addEventListener("input", () => {
  const input = $input.value;
  const { chance_ai, chance_human, metrics } = predict(input);

  $output.innerText = `Chance of AI = ${chance_ai}%
Chance of Human = ${chance_human}%

Text is most likely ${chance_ai >= chance_human ? "AI" : "Human"}

${JSON.stringify(metrics, (_, x) => x, 4)}`;
});
