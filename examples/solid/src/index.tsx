/* @refresh reload */
import { render } from "solid-js/web";
import { Component } from "solid-js";
import { createClient, FetchTransport } from "@rspc/client";
import type { Operations } from "./ts/index"; // Import bindings generated by Rust

const client = createClient<Operations>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const App: Component = () => {
  client.query("version").then((v) => console.log(v));
  client.query("getUser", 1).then((v) => console.log(v));

  return <h1>Hello World</h1>;
};

render(() => <App />, document.getElementById("root") as HTMLElement);