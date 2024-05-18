'use client'
import styles from "./page.module.css";
import { invoke } from "@tauri-apps/api/tauri"
import { useEffect } from "react";
import Table from "./genericTable/layout";
import Image from "next/image";


const colsTestJson = [
  "id",
  "name",
  "age"
];

const rowsTestJson = [
  {
    "id": 1,
    "name": "John",
    "age": 30
  },
  {
    "id": 2,
    "name": "Doe",
    "age": 25
  },
  {
    "id": 3,
    "name": "Jane",
    "age": 28
  },
  {
    "id": 4,
    "name": "Doe",
    "age": 25
  }

];

export default function Home() {

  useEffect(() => {
    const process_data = invoke("get_process_wrapper").then((res) => {
      console.log(res);
    });

  });

  return (
    <main className={styles.main}>
      <div className={styles.description}>
        <Image src="/RustySnout.png" alt="Next.js Logo" width={50} height={50} className={styles.logo}/>
        <p>RustySnout</p>
      </div>
        <Table rows={rowsTestJson} columns={colsTestJson} />
    </main>
  );
}

      // <div className={styles.description}>
      //   <p>
      //     Get started by editing&nbsp;
      //     <code className={styles.code}>src/app/page.js</code>
      //   </p>
      //   <div>
      //     <a
      //       href="https://vercel.com?utm_source=create-next-app&utm_medium=appdir-template&utm_campaign=create-next-app"
      //       target="_blank"
      //       rel="noopener noreferrer"
      //     >
      //       By{" "}
      //       <Image
      //         src="/vercel.svg"
      //         alt="Vercel Logo"
      //         className={styles.vercelLogo}
      //         width={100}
      //         height={24}
      //         priority
      //       />
      //     </a>
      //   </div>
      // </div>

      // <div className={styles.center}>
      //   <Image
      //     className={styles.logo}
      //     src="/next.svg"
      //     alt="Next.js Logo"
      //     width={180}
      //     height={37}
      //     priority
      //   />
      // </div>

      // <div className={styles.grid}>
      //   <a
      //     href="https://nextjs.org/docs?utm_source=create-next-app&utm_medium=appdir-template&utm_campaign=create-next-app"
      //     className={styles.card}
      //     target="_blank"
      //     rel="noopener noreferrer"
      //   >
      //     <h2>
      //       Docs <span>-&gt;</span>
      //     </h2>
      //     <p>Find in-depth information about Next.js features and API.</p>
      //   </a>

      //   <a
      //     href="https://nextjs.org/learn?utm_source=create-next-app&utm_medium=appdir-template&utm_campaign=create-next-app"
      //     className={styles.card}
      //     target="_blank"
      //     rel="noopener noreferrer"
      //   >
      //     <h2>
      //       Learn <span>-&gt;</span>
      //     </h2>
      //     <p>Learn about Next.js in an interactive course with&nbsp;quizzes!</p>
      //   </a>

      //   <a
      //     href="https://vercel.com/templates?framework=next.js&utm_source=create-next-app&utm_medium=appdir-template&utm_campaign=create-next-app"
      //     className={styles.card}
      //     target="_blank"
      //     rel="noopener noreferrer"
      //   >
      //     <h2>
      //       Templates <span>-&gt;</span>
      //     </h2>
      //     <p>Explore starter templates for Next.js.</p>
      //   </a>

      //   <a
      //     href="https://vercel.com/new?utm_source=create-next-app&utm_medium=appdir-template&utm_campaign=create-next-app"
      //     className={styles.card}
      //     target="_blank"
      //     rel="noopener noreferrer"
      //   >
      //     <h2>
      //       Deploy <span>-&gt;</span>
      //     </h2>
      //     <p>
      //       Instantly deploy your Next.js site to a shareable URL with Vercel.
      //     </p>
      //   </a>
      // </div>