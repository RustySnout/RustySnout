'use client';
import React, { useState, useEffect } from 'react';
import Table from "../genericTable/layout";
import Graph from "../graph/layout";
import ControlPanel from '../ControlPanel/layout'; // Correct the import path
import styles from './layout.module.css';
import { invoke } from "@tauri-apps/api/tauri";
import Image from "next/image";

const MonitoringView = () => {
  const [colHeaders, setColHeaders] = useState([]);
  const [rowData, setRowData] = useState([]);
  const [invokeFunction, setInvokeFunction] = useState("get_connections_wrapper");
  const [refresh, setRefresh] = useState(false);  

  // Button handlers that change the state of invokeFunction to the tauri command we need
  // These are called onClick in the html below
  const handleProcess = () => {
    setInvokeFunction("get_connections_wrapper");
  };

  const handleConnection = () => { 
    setInvokeFunction("get_process_wrapper");
  };

  const handleRemoteAddr = () => { 
    setInvokeFunction("get_remote_address_wrapper");
  };

  useEffect(() => {
    if (!showControlPanel) { // Fetch data only if ControlPanel is not shown
      setRefresh(true);
      const fetchData = () => {
        invoke(invokeFunction).then((res) => {
          setRefresh(false);
          console.log(res);
          const data = JSON.parse(res);
          setColHeaders(Object.keys(data[0]));
          setRowData(data);
          // Assuming UpBPS is part of the data fetched
          if (data.length > 0 && data[0].UpBPS !== undefined) {
            setUpBPS(data[0].UpBPS);
          }
        }).catch((err) => {
          setRefresh(false);
          console.log(err);
        });
      };

      // Fetch data every 5 seconds
      fetchData();
      const intervalId = setInterval(fetchData, 5000);
      return () => clearInterval(intervalId);
    }
  }, [invokeFunction, showControlPanel]);

  // This changes the style of the refresh icon to show that the data is being refreshed
  // See layout.module.css for the styles .refreshing and .refresh to see how it is done
  const refreshStyle = refresh ? styles.refreshing : styles.refresh;

  return (
    <div className={styles.monitoringBody}>
      <div className={styles.topContainer}>
        <div className={styles.buttonContainer}>
          <button onClick={handleProcess}> 
            <Image src="/process.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button onClick={handleConnection}> 
            <Image src="/connections.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button onClick={handleRemoteAddr}> 
            <Image src="/remoteAddr.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>
          </button>
          <button onClick={handleControl}> 
            <Image src="/control.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/> 
          </button>
        </div>
        <Graph UpBPS={UpBPS} /> {/* Pass UpBPS to Graph component */}
      </div>
      <div className={styles.refreshTable}>
        {showControlPanel ? (
          <ControlPanel onClose={handleCloseControlPanel} /> // Render ControlPanel if showControlPanel is true
        ) : (
          <>
            <Image src="/loading.svg" alt="Next.js Logo" width={50} height={50} className={refreshStyle}/>
            <Table rows={rowData} columns={colHeaders} />
          </>
        )}
      </div>
    </div>
  );
};

export default MonitoringView;
