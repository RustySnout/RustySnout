
import React from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import styles from './layout.module.css';

const data = [
  { name: 'Ahmed', uv: 4000, pv: 2400, amt: 2400 },
  { name: 'Waseem', uv: 3000, pv: 1398, amt: 2210 },
  { name: 'Mohammed', uv: 2000, pv: 9800, amt: 2290 },
  { name: 'Adly', uv: 2780, pv: 3908, amt: 2000 },
  { name: 'Raslan', uv: 1890, pv: 2000, amt: 2181 },
  { name: 'and', uv: 2390, pv: 3800, amt: 2500 },
  { name: 'Se7s', uv: 10000, pv: 12000, amt: 1000 },
];

const GraphBody = () => {
  return (
    <div className={styles.graphBody}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          width={500}
          height={400}
          data={data}
          margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
        >
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="name" />
          <YAxis />
          <Tooltip />
          <Area type="monotone" dataKey="uv" stackId="1" stroke="#8884d8" fill="#8884d8" />
          <Area type="monotone" dataKey="pv" stackId="1" stroke="#82ca9d" fill="#82ca9d" />
          <Area type="monotone" dataKey="amt" stackId="1" stroke="#ffc658" fill="#ffc658" />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};


const Graph = () => {
  return (
    <div className={styles.graphBody}>
      <GraphBody />
    </div>
  );
};

export default Graph;

