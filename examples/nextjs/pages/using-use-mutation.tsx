import { NextPage } from "next";
import Head from "next/head";
import { rspc } from "../src/rspc";
import styles from "../styles/Home.module.css";

const UsingUseMutation: NextPage = () => {
  const { mutate, data, isLoading, error } = rspc.useMutation("sendMsg");

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    mutate(event.currentTarget.message.value);
  };

  return (
    <div className={styles.container}>
      <Head>
        <title>Using useMutation | RSPC Example with Next.js</title>
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>
          <code>useMutation</code>
        </h1>

        <form onSubmit={handleSubmit}>
          <input
            type="text"
            name="message"
            placeholder="Your message"
            defaultValue="Hello from the client!"
          />
          <button>Submit</button>
        </form>

        <p className={styles.description}>
          {isLoading && "Loading data ..."}
          {data && `Server received message: ${data}`}
          {error && JSON.stringify(error)}
        </p>
      </main>
    </div>
  );
};

export default UsingUseMutation;
