import { predict } from "./pkg/how_much_of_som_is_ai";
import './style.css'

const $input = document.getElementById("input") as HTMLTextAreaElement;
const $output = document.getElementById("output") as HTMLPreElement;

$input.addEventListener("input", () => {
  const input = $input.value;
  const { percent_human, percent_ai, ai, metrics } = predict(input);

  $output.innerText = `Chance of AI = ${percent_human}%
Chance of Human = ${percent_ai}%

Text is most likely ${ai ? "AI" : "Human"}

${JSON.stringify(metrics, (_, x) => x, 4)}`;
});
