import { GoogleGenAI } from "@google/genai";

const apiKey = process.env.GEMINI_API_KEY;

export const generateAgentResponse = async (agentName: string, agentRole: string, userMessage: string, history: { role: 'user' | 'model', parts: string }[]) => {
  if (!apiKey) {
    throw new Error("GEMINI_API_KEY is not set");
  }

  const ai = new GoogleGenAI({ apiKey });
  
  const systemInstruction = `You are ${agentName}. ${agentRole}. 
  You are participating in a collaborative workspace chat. 
  Keep your responses concise and professional, matching the tone of a senior engineer or specialist.
  Use markdown for formatting if needed.`;

  const response = await ai.models.generateContent({
    model: "gemini-3-flash-preview",
    contents: [
      ...history.map(h => ({ role: h.role, parts: [{ text: h.parts }] })),
      { role: 'user', parts: [{ text: userMessage }] }
    ],
    config: {
      systemInstruction,
      temperature: 0.7,
    },
  });

  return response.text;
};
