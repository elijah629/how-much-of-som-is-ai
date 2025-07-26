import { predict } from "./pkg/how_much_of_som_is_ai";
import './style.css'

const $input = document.getElementById("input") as HTMLTextAreaElement;
const $output = document.getElementById("output") as HTMLSpanElement;

$input.addEventListener("input", () => {
  const input = $input.value;
  const prediction = predict(input);

  $output.innerText = prediction ? "Text is AI" : "Text is not AI";
});
